use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
struct AppState {
    db_pool: sqlx::SqlitePool, // SQLx pools are already Clone and thread-safe
}

// Request/Response structures
#[derive(Debug, Deserialize)]
struct SearchRequest {
    pattern: String,
    #[serde(default = "default_node_type")]
    node_type: String,
}

fn default_node_type() -> String {
    "all".to_string()
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    success: bool,
    count: usize,
    nodes: Vec<NodeInfo>,
}

#[derive(Debug, Serialize)]
struct NodeInfo {
    id: String,
    node_type: String,
}

#[derive(Debug, Deserialize)]
struct ContextRequest {
    node_id: String,
    #[serde(default = "default_max_depth")]
    max_depth: u32,
    #[serde(default = "default_max_tokens")]
    max_tokens: usize,
}

fn default_max_depth() -> u32 {
    2
}

fn default_max_tokens() -> usize {
    2000
}

#[derive(Debug, Serialize)]
struct ContextResponse {
    success: bool,
    node_id: String,
    context_items: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DependencyRequest {
    node_id: String,
    #[serde(default = "default_dep_depth")]
    depth: u32,
}

fn default_dep_depth() -> u32 {
    3
}

#[derive(Debug, Serialize)]
struct DependencyResponse {
    success: bool,
    node_id: String,
    depth: u32,
    context_items: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FilesRequest {}

#[derive(Debug, Serialize)]
struct FilesResponse {
    success: bool,
    count: usize,
    files: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db_url = "sqlite://graphrag.db";
    let pool = sqlx::SqlitePool::connect(db_url).await?;
    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route("/search", post(search_nodes))
        .route("/context", post(get_node_context))
        .route("/dependencies", post(get_dependency_graph))
        .route("/files", post(list_all_files))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Code Graph API Server running on http://localhost:3000");
    println!("\nAvailable endpoints:");
    println!("  POST /search        - Search for nodes by name");
    println!("  POST /context       - Get node context with dependencies");
    println!("  POST /dependencies  - Get dependency graph");
    println!("  POST /files         - List all parsed files");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn search_nodes(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    match code_graph_tool::db::search_nodes_by_name(&state.db_pool, &req.pattern).await {
        Ok(matches) => {
            let filtered: Vec<_> = if req.node_type == "all" {
                matches
            } else {
                matches
                    .into_iter()
                    .filter(|(_, t)| t == &req.node_type)
                    .collect()
            };

            Ok(Json(SearchResponse {
                success: true,
                count: filtered.len(),
                nodes: filtered
                    .iter()
                    .map(|(id, t)| NodeInfo {
                        id: id.clone(),
                        node_type: t.clone(),
                    })
                    .collect(),
            }))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_node_context(
    State(state): State<AppState>,
    Json(req): Json<ContextRequest>,
) -> Result<Json<ContextResponse>, (StatusCode, String)> {
    match code_graph_tool::db::get_dependency_context(
        &state.db_pool,
        &req.node_id,
        req.max_depth,
        req.max_tokens,
    )
    .await
    {
        Ok(context) => Ok(Json(ContextResponse {
            success: true,
            node_id: req.node_id,
            context_items: context,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_dependency_graph(
    State(state): State<AppState>,
    Json(req): Json<DependencyRequest>,
) -> Result<Json<DependencyResponse>, (StatusCode, String)> {
    match code_graph_tool::db::get_dependency_context(
        &state.db_pool,
        &req.node_id,
        req.depth,
        10000,
    )
    .await
    {
        Ok(context) => Ok(Json(DependencyResponse {
            success: true,
            node_id: req.node_id,
            depth: req.depth,
            context_items: context,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn list_all_files(
    State(state): State<AppState>,
    _req: Json<FilesRequest>,
) -> Result<Json<FilesResponse>, (StatusCode, String)> {
    match code_graph_tool::db::get_all_files(&state.db_pool).await {
        Ok(files) => Ok(Json(FilesResponse {
            success: true,
            count: files.len(),
            files,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
