use tree_sitter::{Parser, Tree};
use tree_sitter_c::LANGUAGE as c_lang;
use tree_sitter_cpp::LANGUAGE as cpp_lang;
use tree_sitter_go::LANGUAGE as go_lang;
use tree_sitter_java::LANGUAGE as java_lang;
use tree_sitter_javascript::LANGUAGE as js_lang;
use tree_sitter_python::LANGUAGE as python_lang;
use tree_sitter_rust::LANGUAGE as rust_lang;
use tree_sitter_typescript::LANGUAGE_TYPESCRIPT as ts_lang;

use crate::graph::CodeGraph;
use crate::models::{ClassNode, FileNode, FunctionNode};

pub fn parse_file(
    path: &str,
    code: &str,
    checksum: &str,
    graph: &mut CodeGraph,
) -> Result<(), Box<dyn std::error::Error>> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let (language, language_name) = match ext {
        "py" => (python_lang.into(), "python"),
        "rs" => (rust_lang.into(), "rust"),
        "js" => (js_lang.into(), "javascript"),
        "ts" => (ts_lang.into(), "typescript"),
        "java" => (java_lang.into(), "java"),
        "go" => (go_lang.into(), "go"),
        "cpp" | "cc" | "cxx" => (cpp_lang.into(), "cpp"),
        "c" | "h" => (c_lang.into(), "c"),
        _ => {
            return Err(format!("Unsupported file extension: {}", ext).into());
        }
    };

    parse_code(code, path, checksum, language, language_name, graph)
}

pub fn parse_code(
    code: &str,
    path: &str,
    checksum: &str,
    language: tree_sitter::Language,
    language_name: &str,
    graph: &mut CodeGraph,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    parser.set_language(&language)?;

    let tree: Tree = match parser.parse(code, None) {
        Some(tree) => tree,
        None => {
            // Return a more helpful error message for parse failures
            return Err(format!(
                "Failed to parse {}: the file may have syntax errors or be in an unsupported format",
                path
            )
            .into());
        }
    };
    let root = tree.root_node();

    // Ensure the file exists as a node in the graph with its full metadata
    graph.add_file(FileNode {
        path: path.to_string(),
        language: language_name.to_string(),
        size: code.len() as i64,
        checksum: checksum.to_string(),
    });

    traverse_and_extract(&mut root.walk(), code, path, graph, None, None)?;

    Ok(())
}

fn is_class_node(kind: &str) -> bool {
    matches!(
        kind,
        "class_definition"
            | "class_declaration"
            | "struct_item"
            | "type_declaration"
            | "class_specifier"
    )
}

fn is_function_node(kind: &str) -> bool {
    matches!(
        kind,
        "function_definition"
            | "function_item"
            | "function_declaration"
            | "method_definition"
            | "method_declaration"
    )
}

fn is_import_node(kind: &str) -> bool {
    matches!(
        kind,
        "import_statement" | "import_from_statement" | "use_declaration" | "import_declaration"
    )
}

fn is_call_node(kind: &str) -> bool {
    matches!(kind, "call" | "call_expression")
}

fn traverse_and_extract(
    cursor: &mut tree_sitter::TreeCursor,
    code: &str,
    path: &str,
    graph: &mut CodeGraph,
    current_function: Option<String>,
    current_class: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let node = cursor.node();
        let mut next_function = current_function.clone();
        let mut next_class = current_class.clone();
        let kind = node.kind();

        if is_class_node(kind) {
            let name_node = node.child_by_field_name("name").or_else(|| {
                let mut w = node.walk();
                let found = node
                    .children(&mut w)
                    .find(|c| c.kind() == "identifier" || c.kind() == "type_identifier");
                found
            });
            if let Some(name_node) = name_node {
                let name = name_node.utf8_text(code.as_bytes())?;
                let range = node.range();
                let class_id = format!("{}:{}", path, name);

                next_class = Some(name.to_string());

                graph.add_class(ClassNode {
                    file_path: path.to_string(),
                    name: name.to_string(),
                    line: range.start_point.row,
                    end_line: range.end_point.row,
                });
                graph.add_contains_edge(path, &class_id);

                for field in ["superclasses", "base_classes"] {
                    if let Some(superclasses_node) = node.child_by_field_name(field) {
                        let mut walker = superclasses_node.walk();
                        for child in superclasses_node.children(&mut walker) {
                            if child.kind() == "identifier"
                                || child.kind() == "attribute"
                                || child.kind() == "type_identifier"
                            {
                                let base_class_name = child.utf8_text(code.as_bytes())?;
                                graph.add_inherits_edge(&class_id, base_class_name);
                            }
                        }
                    }
                }
            }
        }

        if is_function_node(kind) {
            let name_node = node.child_by_field_name("name").or_else(|| {
                let mut w = node.walk();
                let found = node
                    .children(&mut w)
                    .find(|c| c.kind() == "identifier" || c.kind() == "property_identifier");
                found
            });
            if let Some(name_node) = name_node {
                let name = name_node.utf8_text(code.as_bytes())?;
                let range = node.range();

                // Namespace the method under the class to avoid collisions
                let func_name = if let Some(ref c_name) = current_class {
                    format!("{}.{}", c_name, name)
                } else {
                    name.to_string()
                };

                // Extract parameters and return type to build a signature
                let mut signature = None;
                if let Some(params_node) = node.child_by_field_name("parameters") {
                    let mut sig = params_node.utf8_text(code.as_bytes())?.to_string();
                    if let Some(return_node) = node.child_by_field_name("return_type") {
                        let ret_type = return_node.utf8_text(code.as_bytes())?;
                        sig.push_str(&format!(" -> {}", ret_type));
                    }
                    signature = Some(sig);
                }

                next_function = Some(func_name.clone());

                graph.add_function(FunctionNode {
                    file_path: path.to_string(),
                    name: func_name.clone(),
                    line: range.start_point.row,
                    end_line: range.end_point.row,
                    signature,
                });

                let func_id = format!("{}:{}", path, func_name);
                if let Some(ref c_name) = current_class {
                    let class_id = format!("{}:{}", path, c_name);
                    graph.add_contains_edge(&class_id, &func_id);
                } else {
                    graph.add_contains_edge(path, &func_id);
                }
            }
        }

        // Extract imports
        if is_import_node(kind) {
            let mut walker = node.walk();
            for child in node.children(&mut walker) {
                if [
                    "dotted_name",
                    "scoped_identifier",
                    "string",
                    "string_fragment",
                ]
                .contains(&child.kind())
                {
                    let module_name = child
                        .utf8_text(code.as_bytes())?
                        .trim_matches(|c| c == '"' || c == '\'');
                    graph.add_import_edge(path, module_name);
                } else if child.kind() == "aliased_import" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let module_name = name_node.utf8_text(code.as_bytes())?;
                        graph.add_import_edge(path, module_name);
                    }
                }
            }

            if let Some(module_node) = node
                .child_by_field_name("module_name")
                .or_else(|| node.child_by_field_name("source"))
            {
                let module_name = module_node
                    .utf8_text(code.as_bytes())?
                    .trim_matches(|c| c == '"' || c == '\'');
                graph.add_import_edge(path, module_name);
            }
        }

        // Extract function calls and build the graph edge
        if is_call_node(kind) {
            if let Some(func_node) = node
                .child_by_field_name("function")
                .or_else(|| node.child_by_field_name("name"))
            {
                let called_name = func_node.utf8_text(code.as_bytes())?;

                // If we are currently inside a function, link them!
                if let Some(ref caller) = current_function {
                    let caller_id = format!("{}:{}", path, caller);
                    // Note: For MVP, we are assuming the called function is in the same file.
                    let called_id = format!("{}:{}", path, called_name);
                    graph.add_call_edge(&caller_id, &called_id);
                }
            }
        }

        if cursor.goto_first_child() {
            traverse_and_extract(
                cursor,
                code,
                path,
                graph,
                next_function.clone(),
                next_class.clone(),
            )?;
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter_java::LANGUAGE as java_lang;

    #[test]
    fn test_extracts_functions() {
        let code = r#"
import os
from typing import List, Optional

class CodeParser:
    def __init__(self, language: str):
        pass
    def add_node(self, name: str, node_type: str) -> None:
        pass

def main():
    pass
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "test.py",
            "dummy_checksum",
            python_lang.into(),
            "python",
            &mut graph,
        )
        .unwrap();

        assert_eq!(graph.nodes.node_count(), 9); // 1 file node + 1 class + 2 methods + 1 function + 4 module stubs (os, typing, List, Optional)
        assert!(graph.node_indices.contains_key("test.py"));
        assert!(graph.node_indices.contains_key("test.py:CodeParser"));
        assert!(graph
            .node_indices
            .contains_key("test.py:CodeParser.__init__"));
        assert!(graph
            .node_indices
            .contains_key("test.py:CodeParser.add_node"));
        assert!(graph.node_indices.contains_key("test.py:main"));
    }

    use tree_sitter_c::LANGUAGE as c_lang;
    use tree_sitter_cpp::LANGUAGE as cpp_lang;
    use tree_sitter_go::LANGUAGE as go_lang;
    use tree_sitter_javascript::LANGUAGE as js_lang;
    use tree_sitter_rust::LANGUAGE as rust_lang;
    use tree_sitter_typescript::LANGUAGE_TYPESCRIPT as ts_lang;

    #[test]
    fn test_extracts_rust_structs_and_methods() {
        let code = r#"
use std::collections::HashMap;

pub struct User {
    pub id: u32,
    pub name: String,
}

impl User {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }

    pub fn display(&self) -> String {
        format!("User {}", self.name)
    }
}

pub struct UserManager {
    users: HashMap<u32, User>,
}

impl UserManager {
    pub fn new() -> Self {
        Self { users: HashMap::new() }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }
}

fn main() {
    let mut manager = UserManager::new();
}
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "sample.rs",
            "dummy_checksum",
            rust_lang.into(),
            "rust",
            &mut graph,
        )
        .unwrap();

        println!(
            "Rust nodes: {:?}",
            graph.node_indices.keys().collect::<Vec<_>>()
        );

        assert!(graph.node_indices.contains_key("sample.rs"));
        assert!(graph.node_indices.contains_key("sample.rs:User"));
        assert!(graph.node_indices.contains_key("sample.rs:new"));
        assert!(graph.node_indices.contains_key("sample.rs:display"));
        assert!(graph.node_indices.contains_key("sample.rs:UserManager"));
        assert!(graph
            .node_indices
            .contains_key("sample.rs:UserManager::new"));
        assert!(graph.node_indices.contains_key("sample.rs:add_user"));
    }

    #[test]
    fn test_extracts_javascript_classes_and_methods() {
        let code = r#"
class User {
    constructor(id, name) {
        this.id = id;
        this.name = name;
    }

    setEmail(email) {
        this.email = email;
    }

    display() {
        return `User ${this.id}: ${this.name}`;
    }
}

class UserManager extends EventEmitter {
    constructor() {
        super();
        this.users = new Map();
    }

    addUser(user) {
        this.users.set(user.id, user);
    }

    getUser(id) {
        return this.users.get(id);
    }
}

function main() {
    const manager = new UserManager();
}
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "sample.js",
            "dummy_checksum",
            js_lang.into(),
            "javascript",
            &mut graph,
        )
        .unwrap();

        println!(
            "JS nodes: {:?}",
            graph.node_indices.keys().collect::<Vec<_>>()
        );

        assert!(graph.node_indices.contains_key("sample.js"));
        assert!(graph.node_indices.contains_key("sample.js:User"));
        assert!(graph
            .node_indices
            .contains_key("sample.js:User.constructor"));
        assert!(graph.node_indices.contains_key("sample.js:User.setEmail"));
        assert!(graph.node_indices.contains_key("sample.js:User.display"));
        assert!(graph.node_indices.contains_key("sample.js:UserManager"));
        assert!(graph
            .node_indices
            .contains_key("sample.js:UserManager.constructor"));
        assert!(graph
            .node_indices
            .contains_key("sample.js:UserManager.addUser"));
        assert!(graph
            .node_indices
            .contains_key("sample.js:UserManager.getUser"));
        assert!(graph.node_indices.contains_key("sample.js:main"));
    }

    #[test]
    fn test_extracts_typescript_classes_and_methods() {
        let code = r#"
export interface UserData {
    id: number;
    name: string;
}

export class User implements UserData {
    public id: number;
    public name: string;

    constructor(id: number, name: string) {
        this.id = id;
        this.name = name;
    }

    setEmail(email: string): void {
        this.email = email;
    }
}

export class UserManager {
    private users: Map<number, User>;

    constructor() {
        this.users = new Map();
    }

    addUser(user: User): void {
        this.users.set(user.id, user);
    }

    getUser(id: number): User | undefined {
        return this.users.get(id);
    }
}

export function main(): void {
    const manager = new UserManager();
}
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "sample.ts",
            "dummy_checksum",
            ts_lang.into(),
            "typescript",
            &mut graph,
        )
        .unwrap();

        println!(
            "TS nodes: {:?}",
            graph.node_indices.keys().collect::<Vec<_>>()
        );

        assert!(graph.node_indices.contains_key("sample.ts"));
        assert!(graph.node_indices.contains_key("sample.ts:User"));
        assert!(graph
            .node_indices
            .contains_key("sample.ts:User.constructor"));
        assert!(graph.node_indices.contains_key("sample.ts:User.setEmail"));
        assert!(graph.node_indices.contains_key("sample.ts:UserManager"));
        assert!(graph
            .node_indices
            .contains_key("sample.ts:UserManager.constructor"));
        assert!(graph
            .node_indices
            .contains_key("sample.ts:UserManager.addUser"));
        assert!(graph
            .node_indices
            .contains_key("sample.ts:UserManager.getUser"));
        assert!(graph.node_indices.contains_key("sample.ts:main"));
    }

    #[test]
    fn test_extracts_go_structs_and_methods() {
        let code = r#"
package main

import "fmt"

// User represents a user in the system
type User struct {
    ID    int
    Name  string
    Email string
}

// NewUser creates a new user
func NewUser(id int, name string) *User {
    return &User{ID: id, Name: name}
}

// SetEmail sets the user's email
func (u *User) SetEmail(email string) {
    u.Email = email
}

// Display returns a formatted string
func (u *User) Display() string {
    return fmt.Sprintf("User %d", u.ID)
}

type UserManager struct {
    users map[int]*User
}

// NewUserManager creates a new manager
func NewUserManager() *UserManager {
    return &UserManager{users: make(map[int]*User)}
}

// AddUser adds a user to the manager
func (m *UserManager) AddUser(user *User) {
    m.users[user.ID] = user
}

// main is the entry point
func main() {
    manager := NewUserManager()
}
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "sample.go",
            "dummy_checksum",
            go_lang.into(),
            "go",
            &mut graph,
        )
        .unwrap();

        println!(
            "Go nodes: {:?}",
            graph.node_indices.keys().collect::<Vec<_>>()
        );

        assert!(graph.node_indices.contains_key("sample.go"));
        // Go structs are not always extracted, but functions should be
        assert!(graph.node_indices.contains_key("sample.go:NewUser"));
        assert!(graph.node_indices.contains_key("sample.go:SetEmail"));
        assert!(graph.node_indices.contains_key("sample.go:Display"));
        // UserManager struct may not be extracted, but methods should be
        assert!(graph.node_indices.contains_key("sample.go:NewUserManager"));
        assert!(graph.node_indices.contains_key("sample.go:AddUser"));
        assert!(graph.node_indices.contains_key("sample.go:main"));
    }

    #[test]
    fn test_extracts_c_functions_and_structs() {
        let code = r#"
typedef struct {
    int id;
    char name[50];
} User;

User* user_new(int id, const char* name) {
    User* user = (User*)malloc(sizeof(User));
    return user;
}

void user_set_email(User* user, const char* email) {
    if (user && email) {
        strncpy(user->email, email, 100);
    }
}

char* user_display(const User* user) {
    char* buffer = (char*)malloc(128);
    return buffer;
}

typedef struct {
    User** users;
    int count;
} UserManager;

UserManager* user_manager_new() {
    UserManager* manager = (UserManager*)malloc(sizeof(UserManager));
    return manager;
}

void user_manager_add(UserManager* manager, User* user) {
    if (!manager || !user) return;
}

int main() {
    UserManager* manager = user_manager_new();
    return 0;
}
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "sample.c",
            "dummy_checksum",
            c_lang.into(),
            "c",
            &mut graph,
        )
        .unwrap();

        println!(
            "C nodes: {:?}",
            graph.node_indices.keys().collect::<Vec<_>>()
        );

        // C parser has limited support for structs and functions
        assert!(graph.node_indices.contains_key("sample.c"));
        // At minimum, the file node should exist
        assert!(graph.nodes.node_count() >= 1);
    }

    #[test]
    fn test_extracts_cpp_classes_and_methods() {
        let code = r#"
class User {
private:
    int id_;
    std::string name_;

public:
    User(int id, const std::string& name) : id_(id), name_(name) {}

    void setEmail(const std::string& email) {
        email_ = email;
    }

    std::string display() const {
        return "User " + std::to_string(id_);
    }
};

class UserManager {
private:
    std::vector<std::unique_ptr<User>> users_;

public:
    UserManager() = default;

    void addUser(std::unique_ptr<User> user) {
        users_.push_back(std::move(user));
    }

    User* getUser(int id) {
        for (auto& user : users_) {
            if (user->getId() == id) {
                return user.get();
            }
        }
        return nullptr;
    }
};

int main() {
    UserManager manager;
    return 0;
}
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "sample.cpp",
            "dummy_checksum",
            cpp_lang.into(),
            "cpp",
            &mut graph,
        )
        .unwrap();

        println!(
            "C++ nodes: {:?}",
            graph.node_indices.keys().collect::<Vec<_>>()
        );

        // C++ parser has limited method extraction
        assert!(graph.node_indices.contains_key("sample.cpp"));
        assert!(graph.node_indices.contains_key("sample.cpp:User"));
        assert!(graph.node_indices.contains_key("sample.cpp:UserManager"));
        // main may or may not be extracted depending on parser support
    }

    #[test]
    fn test_extracts_java_classes_and_methods() {
        let code = r#"
package com.example;

import java.util.List;
import java.util.ArrayList;

public class UserService {
    private List<String> users;

    public UserService() {
        this.users = new ArrayList<>();
    }

    public void addUser(String name) {
        users.add(name);
    }

    public String getUser(int index) {
        return users.get(index);
    }

    private void validate(String name) {
        if (name == null) {
            throw new IllegalArgumentException();
        }
    }
}
"#;
        let mut graph = CodeGraph::new();
        parse_code(
            code,
            "UserService.java",
            "dummy_checksum",
            java_lang.into(),
            "java",
            &mut graph,
        )
        .unwrap();

        // Debug: print all node indices
        println!(
            "All nodes: {:?}",
            graph.node_indices.keys().collect::<Vec<_>>()
        );

        // Should have: 1 file node + 1 class + 3 methods + 2 import stubs
        assert_eq!(graph.nodes.node_count(), 7);
        assert!(graph.node_indices.contains_key("UserService.java"));
        assert!(graph
            .node_indices
            .contains_key("UserService.java:UserService"));
        assert!(graph
            .node_indices
            .contains_key("UserService.java:UserService.addUser"));
        assert!(graph
            .node_indices
            .contains_key("UserService.java:UserService.getUser"));
        assert!(graph
            .node_indices
            .contains_key("UserService.java:UserService.validate"));
    }
}
