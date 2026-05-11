//! Integration tests for the code graph tool
//!
//! These tests verify the full flow from parsing to query execution.
//!
//! Integration tests are in a separate crate and can only access
//! the public API of the library, simulating real-world usage.

use code_graph_tool::db;
use code_graph_tool::graph::CodeGraph;
use code_graph_tool::parser;
use sqlx::SqlitePool;
use tree_sitter_python::LANGUAGE as python_lang;

// Helper to create a test database with proper schema
async fn create_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    // Initialize schema inline since init_db creates its own connection
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL,
            name TEXT,
            file_path TEXT,
            language TEXT,
            size INTEGER,
            line INTEGER,
            end_line INTEGER,
            signature TEXT,
            checksum TEXT
        );

        CREATE TABLE IF NOT EXISTS edges (
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            relation TEXT NOT NULL,
            PRIMARY KEY (source_id, target_id, relation),
            FOREIGN KEY (source_id) REFERENCES nodes(id),
            FOREIGN KEY (target_id) REFERENCES nodes(id)
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn test_full_parsing_and_querying() {
    let pool = create_test_db().await;

    // Create a test graph and parse code
    let mut graph = CodeGraph::new();
    let code = r#"
import os
from typing import List

def calculate_sum(numbers: List[int]) -> int:
    total = 0
    for num in numbers:
        total += num
    return total

class DataProcessor:
    def __init__(self, name: str):
        self.name = name

    def process(self, data: List[str]) -> List[str]:
        return [d.upper() for d in data]
"#;

    parser::parse_code(
        code,
        "test_integration.py",
        "test_checksum_123",
        python_lang.into(),
        "python",
        &mut graph,
    )
    .unwrap();

    // Verify graph structure
    assert!(graph.node_indices.contains_key("test_integration.py"));
    assert!(graph
        .node_indices
        .contains_key("test_integration.py:calculate_sum"));
    assert!(graph
        .node_indices
        .contains_key("test_integration.py:DataProcessor"));
    assert!(graph
        .node_indices
        .contains_key("test_integration.py:DataProcessor.__init__"));
    assert!(graph
        .node_indices
        .contains_key("test_integration.py:DataProcessor.process"));

    // Save to database
    db::save_graph(&graph, &pool).await.unwrap();

    // Query for context
    let context = db::get_dependency_context(&pool, "test_integration.py:calculate_sum", 2, 5000)
        .await
        .unwrap();

    // Verify context was retrieved
    assert!(!context.is_empty());
    assert!(context[0].contains("calculate_sum"));
}

#[tokio::test]
async fn test_incremental_updates() {
    let pool = create_test_db().await;

    // First parse
    let mut graph1 = CodeGraph::new();
    let code1 = r#"def hello(): pass"#;
    parser::parse_code(
        code1,
        "test.py",
        "checksum_v1",
        python_lang.into(),
        "python",
        &mut graph1,
    )
    .unwrap();
    db::save_graph(&graph1, &pool).await.unwrap();

    // Verify function exists
    let files = db::get_all_files(&pool).await.unwrap();
    assert!(files.contains(&"test.py".to_string()));

    // Update with new code
    let mut graph2 = CodeGraph::new();
    let code2 = r#"
def hello(): pass

def world(): pass
"#;
    parser::parse_code(
        code2,
        "test.py",
        "checksum_v2",
        python_lang.into(),
        "python",
        &mut graph2,
    )
    .unwrap();
    db::save_graph(&graph2, &pool).await.unwrap();

    // Verify both functions exist after update
    let matches = db::search_nodes_by_name(&pool, "hello").await.unwrap();
    assert!(!matches.is_empty());

    let matches = db::search_nodes_by_name(&pool, "world").await.unwrap();
    assert!(!matches.is_empty());
}

#[tokio::test]
async fn test_dependency_traversal() {
    let pool = create_test_db().await;

    // Create graph with dependencies
    let mut graph = CodeGraph::new();

    // Add file nodes
    graph.add_file(code_graph_tool::models::FileNode {
        path: "main.py".to_string(),
        language: "python".to_string(),
        size: 100,
        checksum: "main_checksum".to_string(),
    });

    graph.add_file(code_graph_tool::models::FileNode {
        path: "utils.py".to_string(),
        language: "python".to_string(),
        size: 50,
        checksum: "utils_checksum".to_string(),
    });

    // Add functions
    graph.add_function(code_graph_tool::models::FunctionNode {
        file_path: "main.py".to_string(),
        name: "main".to_string(),
        line: 0,
        end_line: 5,
        signature: Some("main() -> None".to_string()),
    });

    graph.add_function(code_graph_tool::models::FunctionNode {
        file_path: "utils.py".to_string(),
        name: "helper".to_string(),
        line: 0,
        end_line: 3,
        signature: Some("helper() -> int".to_string()),
    });

    // Add call edge (main calls helper)
    graph.add_call_edge("main.py:main", "utils.py:helper");

    db::save_graph(&graph, &pool).await.unwrap();

    // Query dependencies
    let context = db::get_dependency_context(&pool, "main.py:main", 2, 5000)
        .await
        .unwrap();

    // Should include both main and helper
    let context_str = context.join(" ");
    assert!(context_str.contains("main.py:main"));
    assert!(context_str.contains("utils.py:helper"));
}

#[tokio::test]
async fn test_search_functionality() {
    let pool = create_test_db().await;

    let mut graph = CodeGraph::new();

    // Add multiple functions with similar names
    graph.add_function(code_graph_tool::models::FunctionNode {
        file_path: "module1.py".to_string(),
        name: "process_data".to_string(),
        line: 0,
        end_line: 10,
        signature: None,
    });

    graph.add_function(code_graph_tool::models::FunctionNode {
        file_path: "module2.py".to_string(),
        name: "process_file".to_string(),
        line: 0,
        end_line: 5,
        signature: None,
    });

    graph.add_class(code_graph_tool::models::ClassNode {
        file_path: "models.py".to_string(),
        name: "DataProcessor".to_string(),
        line: 0,
        end_line: 20,
    });

    db::save_graph(&graph, &pool).await.unwrap();

    // Search for 'process' - also matches DataProcessor
    let matches = db::search_nodes_by_name(&pool, "process").await.unwrap();
    assert_eq!(matches.len(), 3); // process_data, process_file, and DataProcessor

    // Search for 'Data' - case-insensitive search
    let matches = db::search_nodes_by_name(&pool, "Data").await.unwrap();
    assert!(!matches.is_empty()); // At least DataProcessor

    // Search for non-existent
    let matches = db::search_nodes_by_name(&pool, "nonexistent_xyz")
        .await
        .unwrap();
    assert!(matches.is_empty());
}
