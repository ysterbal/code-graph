use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Code Graph MCP Server");
    println!("=====================");
    println!("Listening for JSON-RPC requests on stdin...");
    println!();

    // Initialize database pool
    let db_url = "sqlite://graphrag.db";
    let pool = code_graph_tool::db::init_db(db_url).await?;

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                send_error(None, format!("Failed to parse request: {}", e));
                continue;
            }
        };

        let response = match request.method.as_str() {
            "initialize" => handle_initialize(request.params),
            "tools/list" => handle_tools_list(),
            "tools/call" => handle_tools_call(request.params, &pool).await,
            _ => {
                send_error(
                    request.id.as_ref(),
                    format!("Unknown method: {}", request.method),
                );
                continue;
            }
        };

        println!("{}", serde_json::to_string(&response)?);
        io::stdout().flush()?;
    }
}

fn handle_initialize(_params: Option<HashMap<String, serde_json::Value>>) -> JsonRpcResponse {
    let result = json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "code-graph-mcp",
            "version": "0.1.0"
        },
        "capabilities": {
            "tools": {}
        }
    });

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: None,
        result: Some(result),
        error: None,
    }
}

fn handle_tools_list() -> JsonRpcResponse {
    let tools = json!([
        {
            "name": "search_nodes",
            "description": "Search for functions, classes, or files by name pattern across the codebase",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Name pattern to search for (supports wildcards like 'main', 'Parser', etc.)"
                    },
                    "node_type": {
                        "type": "string",
                        "enum": ["Function", "Class", "File", "all"],
                        "default": "all",
                        "description": "Filter by node type"
                    }
                },
                "required": ["pattern"]
            }
        },
        {
            "name": "get_node_context",
            "description": "Get detailed context for a specific node including its dependencies and source code",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": {
                        "type": "string",
                        "description": "Full node ID (e.g., 'src/main.rs:main' or 'src/sample.py:CodeParser')"
                    },
                    "max_depth": {
                        "type": "integer",
                        "default": 2,
                        "description": "How many levels of dependencies to include"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "default": 2000,
                        "description": "Maximum tokens for context"
                    }
                },
                "required": ["node_id"]
            }
        },
        {
            "name": "get_dependency_graph",
            "description": "Get all dependencies for a node up to specified depth",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": {
                        "type": "string",
                        "description": "Node ID to get dependencies for"
                    },
                    "depth": {
                        "type": "integer",
                        "default": 3,
                        "description": "Maximum depth of dependency traversal"
                    }
                },
                "required": ["node_id"]
            }
        },
        {
            "name": "list_all_files",
            "description": "List all parsed files in the codebase",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }
    ]);

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: None,
        result: Some(json!({ "tools": tools })),
        error: None,
    }
}

async fn handle_tools_call(
    params: Option<HashMap<String, serde_json::Value>>,
    pool: &SqlitePool,
) -> JsonRpcResponse {
    let (tool_name, args) = if let Some(p) = &params {
        (
            p.get("name").and_then(|v| v.as_str()),
            p.get("arguments").cloned(),
        )
    } else {
        (None, None)
    };

    match tool_name {
        Some("search_nodes") => handle_search_nodes(args, pool).await,
        Some("get_node_context") => handle_get_node_context(args, pool).await,
        Some("get_dependency_graph") => handle_get_dependency_graph(args, pool).await,
        Some("list_all_files") => handle_list_all_files(pool).await,
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: format!("Unknown tool: {:?}", tool_name),
            }),
        },
    }
}

async fn handle_search_nodes(
    args: Option<serde_json::Value>,
    pool: &SqlitePool,
) -> JsonRpcResponse {
    let pattern = args
        .as_ref()
        .and_then(|a| a.get("pattern").and_then(|p| p.as_str()))
        .unwrap_or("");

    let node_type = args
        .as_ref()
        .and_then(|a| a.get("node_type").and_then(|t| t.as_str()))
        .unwrap_or("all");

    match code_graph_tool::db::search_nodes_by_name(pool, pattern).await {
        Ok(matches) => {
            let filtered: Vec<_> = if node_type == "all" {
                matches
            } else {
                matches
                    .into_iter()
                    .filter(|(_, t)| t == node_type)
                    .collect()
            };

            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: Some(json!({
                    "success": true,
                    "count": filtered.len(),
                    "nodes": filtered.iter().map(|(id, t)| json!({
                        "id": id,
                        "type": t
                    })).collect::<Vec<_>>()
                })),
                error: None,
            }
        }
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Search failed: {}", e),
            }),
        },
    }
}

async fn handle_get_node_context(
    args: Option<serde_json::Value>,
    pool: &SqlitePool,
) -> JsonRpcResponse {
    let node_id = args
        .as_ref()
        .and_then(|a| a.get("node_id").and_then(|n| n.as_str()))
        .unwrap_or("");

    let max_depth = args
        .as_ref()
        .and_then(|a| a.get("max_depth").and_then(|d| d.as_i64()))
        .unwrap_or(2) as u32;

    let max_tokens = args
        .as_ref()
        .and_then(|a| a.get("max_tokens").and_then(|t| t.as_i64()))
        .unwrap_or(2000) as usize;

    match code_graph_tool::db::get_dependency_context(pool, node_id, max_depth, max_tokens).await {
        Ok(context) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(json!({
                "success": true,
                "node_id": node_id,
                "context_items": context
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Context fetch failed: {}", e),
            }),
        },
    }
}

async fn handle_get_dependency_graph(
    args: Option<serde_json::Value>,
    pool: &SqlitePool,
) -> JsonRpcResponse {
    let node_id = args
        .as_ref()
        .and_then(|a| a.get("node_id").and_then(|n| n.as_str()))
        .unwrap_or("");

    let depth = args
        .as_ref()
        .and_then(|a| a.get("depth").and_then(|d| d.as_i64()))
        .unwrap_or(3) as u32;

    match code_graph_tool::db::get_dependency_context(pool, node_id, depth, 10000).await {
        Ok(context) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(json!({
                "success": true,
                "node_id": node_id,
                "depth": depth,
                "context_items": context
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Dependency fetch failed: {}", e),
            }),
        },
    }
}

async fn handle_list_all_files(pool: &SqlitePool) -> JsonRpcResponse {
    match code_graph_tool::db::get_all_files(pool).await {
        Ok(files) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(json!({
                "success": true,
                "count": files.len(),
                "files": files
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("File list failed: {}", e),
            }),
        },
    }
}

fn send_error(id: Option<&serde_json::Value>, message: String) {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: id.cloned(),
        result: None,
        error: Some(JsonRpcError {
            code: -32600,
            message,
        }),
    };
    println!("{}", serde_json::to_string(&response).unwrap());
    io::stdout().flush().unwrap();
}
