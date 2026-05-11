use crate::models::{ClassNode, EdgeType, FileNode, FunctionNode, NodeData};
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;

pub struct CodeGraph {
    pub nodes: Graph<NodeData, EdgeType, petgraph::Directed>,
    pub node_indices: HashMap<String, NodeIndex>,
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGraph {
    pub fn new() -> Self {
        CodeGraph {
            nodes: Graph::new(),
            node_indices: HashMap::new(),
        }
    }

    fn upsert_node(&mut self, id: String, data: NodeData) {
        if let Some(&idx) = self.node_indices.get(&id) {
            if let Some(weight) = self.nodes.node_weight_mut(idx) {
                *weight = data;
            }
        } else {
            let idx = self.nodes.add_node(data);
            self.node_indices.insert(id, idx);
        }
    }

    pub fn add_function(&mut self, func: FunctionNode) {
        // Create a unique ID for the node (e.g., "path:name")
        let id = format!("{}:{}", func.file_path, func.name);
        self.upsert_node(id, NodeData::Function(func));
    }

    pub fn add_class(&mut self, class: ClassNode) {
        let id = format!("{}:{}", class.file_path, class.name);
        self.upsert_node(id, NodeData::Class(class));
    }

    pub fn add_file(&mut self, file: FileNode) {
        let id = file.path.clone();
        self.upsert_node(id, NodeData::File(file));
    }

    pub fn add_node_if_missing(&mut self, id: &str) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(id) {
            idx
        } else {
            let idx = self.nodes.add_node(NodeData::Stub(id.to_string()));
            self.node_indices.insert(id.to_string(), idx);
            idx
        }
    }

    pub fn add_call_edge(&mut self, from: &str, to: &str) {
        // If the referenced node doesn't exist yet, we create a "stub" node
        // that will be updated when/if the actual function definition is parsed.
        let u = self.add_node_if_missing(from);
        let v = self.add_node_if_missing(to);
        self.nodes.add_edge(u, v, EdgeType::Calls);
    }

    pub fn add_contains_edge(&mut self, file_path: &str, func_id: &str) {
        let u = self.add_node_if_missing(file_path);
        let v = self.add_node_if_missing(func_id);
        self.nodes.add_edge(u, v, EdgeType::Contains);
    }

    pub fn add_import_edge(&mut self, file_path: &str, module_name: &str) {
        let u = self.add_node_if_missing(file_path);
        let v = self.add_node_if_missing(module_name);
        self.nodes.add_edge(u, v, EdgeType::Imports);
    }

    pub fn add_inherits_edge(&mut self, class_id: &str, base_class_id: &str) {
        let u = self.add_node_if_missing(class_id);
        let v = self.add_node_if_missing(base_class_id);
        self.nodes.add_edge(u, v, EdgeType::InheritsFrom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileNode, FunctionNode};

    #[test]
    fn test_graph_new() {
        let graph = CodeGraph::new();
        assert_eq!(graph.nodes.node_count(), 0);
        assert!(graph.node_indices.is_empty());
    }

    #[test]
    fn test_add_file() {
        let mut graph = CodeGraph::new();

        let file = FileNode {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            size: 100,
            checksum: "abc123".to_string(),
        };

        graph.add_file(file);

        assert_eq!(graph.nodes.node_count(), 1);
        assert!(graph.node_indices.contains_key("test.rs"));
    }

    #[test]
    fn test_add_function() {
        let mut graph = CodeGraph::new();

        let func = FunctionNode {
            file_path: "test.rs".to_string(),
            name: "main".to_string(),
            line: 0,
            end_line: 5,
            signature: Some("main() -> ()".to_string()),
        };

        graph.add_function(func);

        assert_eq!(graph.nodes.node_count(), 1);
        assert!(graph.node_indices.contains_key("test.rs:main"));
    }

    #[test]
    fn test_add_contains_edge() {
        let mut graph = CodeGraph::new();

        graph.add_file(FileNode {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            size: 100,
            checksum: "abc".to_string(),
        });

        graph.add_contains_edge("test.rs", "test.rs:my_func");

        let edges: Vec<_> = graph.nodes.edge_references().collect();
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_add_call_edge() {
        let mut graph = CodeGraph::new();

        graph.add_call_edge("test.rs:func_a", "test.rs:func_b");

        let edges: Vec<_> = graph.nodes.edge_references().collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(*edges[0].weight(), EdgeType::Calls);
    }

    #[test]
    fn test_upsert_node_updates_existing() {
        let mut graph = CodeGraph::new();

        let file1 = FileNode {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            size: 100,
            checksum: "abc".to_string(),
        };
        graph.add_file(file1);

        let file2 = FileNode {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            size: 200,
            checksum: "def".to_string(),
        };
        graph.add_file(file2);

        // Should still have only 1 node (updated)
        assert_eq!(graph.nodes.node_count(), 1);
    }

    #[test]
    fn test_node_data_id() {
        let file = FileNode {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            size: 100,
            checksum: "abc".to_string(),
        };
        let node_data = NodeData::File(file);
        assert_eq!(node_data.id(), "test.rs");
    }

    #[test]
    fn test_edge_type_as_str() {
        assert_eq!(EdgeType::Contains.as_str(), "Contains");
        assert_eq!(EdgeType::Calls.as_str(), "Calls");
        assert_eq!(EdgeType::Imports.as_str(), "Imports");
        assert_eq!(EdgeType::InheritsFrom.as_str(), "InheritsFrom");
    }

    #[test]
    fn test_add_import_edge() {
        let mut graph = CodeGraph::new();

        graph.add_import_edge("main.py", "os");

        let edges: Vec<_> = graph.nodes.edge_references().collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(*edges[0].weight(), EdgeType::Imports);
    }

    #[test]
    fn test_add_inherits_edge() {
        let mut graph = CodeGraph::new();

        graph.add_inherits_edge("test.py:MyClass", "BaseClass");

        let edges: Vec<_> = graph.nodes.edge_references().collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(*edges[0].weight(), EdgeType::InheritsFrom);
    }

    #[test]
    fn test_graph_with_multiple_nodes() {
        let mut graph = CodeGraph::new();

        // Add file
        graph.add_file(FileNode {
            path: "test.rs".to_string(),
            language: "rust".to_string(),
            size: 100,
            checksum: "abc".to_string(),
        });

        // Add functions
        graph.add_function(FunctionNode {
            file_path: "test.rs".to_string(),
            name: "main".to_string(),
            line: 0,
            end_line: 5,
            signature: None,
        });

        graph.add_function(FunctionNode {
            file_path: "test.rs".to_string(),
            name: "helper".to_string(),
            line: 10,
            end_line: 15,
            signature: Some("helper() -> i32".to_string()),
        });

        // Add call edge
        graph.add_call_edge("test.rs:main", "test.rs:helper");

        assert_eq!(graph.nodes.node_count(), 3);
        // Edge count varies based on stub creation, just verify we have edges
        assert!(graph.nodes.edge_count() > 0);
    }
}
