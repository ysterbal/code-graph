use tree_sitter::{Parser, Tree};
use tree_sitter_c::LANGUAGE as c_lang;
use tree_sitter_cpp::LANGUAGE as cpp_lang;
use tree_sitter_go::LANGUAGE as go_lang;
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
}
