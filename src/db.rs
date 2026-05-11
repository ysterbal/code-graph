use crate::graph::CodeGraph;
use crate::models::NodeData;
use petgraph::visit::EdgeRef;
use sqlx::Row;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;
use tracing::info;

/// Estimate token count using a simple heuristic (1 char ≈ 0.25 tokens for English text)
/// For more accurate counting, use tiktoken_rs with proper encoding
fn estimate_tokens(text: &str) -> usize {
    // Simple heuristic: ~4 characters per token for ASCII text
    // This is reasonably accurate for code and English text
    text.chars().count() / 4
}

pub async fn init_db(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    info!("Initializing database at {}", db_url);

    // Ensure the database file is created if it doesn't exist
    let options = SqliteConnectOptions::from_str(db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    info!("Database connection established with max_connections=5");

    // Initialize our graph tables
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
    .await?;

    Ok(pool)
}

pub async fn save_graph(graph: &CodeGraph, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Execute inserts within a transaction for performance
    let mut tx = pool.begin().await?;

    // 1. Persist Nodes
    for idx in graph.nodes.node_indices() {
        let node_data = &graph.nodes[idx];
        let node_id = node_data.id();

        let mut node_type = "Stub";
        let mut name: Option<String> = None;
        let mut file_path: Option<String> = None;
        let mut language: Option<String> = None;
        let mut size: Option<i64> = None;
        let mut line: Option<i64> = None;
        let mut end_line: Option<i64> = None;
        let mut signature: Option<String> = None;
        let mut checksum: Option<String> = None;

        // Extract the specific metadata depending on the Node variant
        match node_data {
            NodeData::File(f) => {
                node_type = "File";
                file_path = Some(f.path.clone());
                language = Some(f.language.clone());
                size = Some(f.size);
                checksum = Some(f.checksum.clone());
            }
            NodeData::Function(f) => {
                node_type = "Function";
                name = Some(f.name.clone());
                file_path = Some(f.file_path.clone());
                line = Some(f.line as i64);
                end_line = Some(f.end_line as i64);
                signature = f.signature.clone();
            }
            NodeData::Class(c) => {
                node_type = "Class";
                name = Some(c.name.clone());
                file_path = Some(c.file_path.clone());
                line = Some(c.line as i64);
                end_line = Some(c.end_line as i64);
            }
            NodeData::Stub(_) => {}
        }

        sqlx
            ::query(
                r#"
                INSERT INTO nodes 
                (id, node_type, name, file_path, language, size, line, end_line, signature, checksum) 
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    node_type = CASE WHEN excluded.node_type != 'Stub' THEN excluded.node_type ELSE nodes.node_type END,
                    name = CASE WHEN excluded.node_type != 'Stub' THEN excluded.name ELSE nodes.name END,
                    file_path = CASE WHEN excluded.node_type != 'Stub' THEN excluded.file_path ELSE nodes.file_path END,
                    language = CASE WHEN excluded.node_type != 'Stub' THEN excluded.language ELSE nodes.language END,
                    size = CASE WHEN excluded.node_type != 'Stub' THEN excluded.size ELSE nodes.size END,
                    line = CASE WHEN excluded.node_type != 'Stub' THEN excluded.line ELSE nodes.line END,
                    end_line = CASE WHEN excluded.node_type != 'Stub' THEN excluded.end_line ELSE nodes.end_line END,
                    signature = CASE WHEN excluded.node_type != 'Stub' THEN excluded.signature ELSE nodes.signature END,
                    checksum = CASE WHEN excluded.node_type != 'Stub' THEN excluded.checksum ELSE nodes.checksum END
                "#
            )
            .bind(&node_id)
            .bind(node_type)
            .bind(name)
            .bind(file_path)
            .bind(language)
            .bind(size)
            .bind(line)
            .bind(end_line)
            .bind(signature)
            .bind(checksum)
            .execute(&mut *tx).await?;
    }

    // 2. Persist Edges
    for edge in graph.nodes.edge_references() {
        let source_id = graph.nodes[edge.source()].id();
        let target_id = graph.nodes[edge.target()].id();
        let relation = edge.weight().as_str(); // Uses static string directly without allocating

        sqlx::query(
            "INSERT OR IGNORE INTO edges (source_id, target_id, relation) VALUES (?, ?, ?)",
        )
        .bind(&source_id)
        .bind(&target_id)
        .bind(relation)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn get_file_checksum(
    pool: &SqlitePool,
    file_path: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query("SELECT checksum FROM nodes WHERE id = ? AND node_type = 'File'")
        .bind(file_path)
        .fetch_optional(pool)
        .await?;

    if let Some(r) = row {
        Ok(r.try_get("checksum")?)
    } else {
        Ok(None)
    }
}

pub async fn get_all_files(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT id FROM nodes WHERE node_type = 'File'")
        .fetch_all(pool)
        .await?;

    let mut files = Vec::new();
    for row in rows {
        files.push(row.try_get("id")?);
    }
    Ok(files)
}

pub async fn search_nodes_by_name(
    pool: &SqlitePool,
    name_pattern: &str,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    // Search for nodes where name matches the pattern (case-insensitive)
    let rows =
        sqlx::query("SELECT id, node_type FROM nodes WHERE name LIKE ? AND node_type != 'File'")
            .bind(format!("%{}%", name_pattern))
            .fetch_all(pool)
            .await?;

    let mut matches = Vec::new();
    for row in rows {
        let id: String = row.try_get("id")?;
        let node_type: String = row.try_get("node_type")?;
        matches.push((id, node_type));
    }
    Ok(matches)
}

pub async fn clear_file_state(pool: &SqlitePool, file_path: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Delete outgoing edges for any node defined in this file
    sqlx::query(
        "DELETE FROM edges WHERE source_id IN (SELECT id FROM nodes WHERE id = ? OR file_path = ?)",
    )
    .bind(file_path)
    .bind(file_path)
    .execute(&mut *tx)
    .await?;

    // 2. Downgrade nodes to 'Stub' to preserve incoming edges from other unchanged files
    sqlx::query(
        r#"
        UPDATE nodes
        SET node_type = 'Stub', name = NULL, file_path = NULL, language = NULL, 
            size = NULL, line = NULL, end_line = NULL, signature = NULL, checksum = NULL
        WHERE id = ? OR file_path = ?
        "#,
    )
    .bind(file_path)
    .bind(file_path)
    .execute(&mut *tx)
    .await?;

    // 3. Clean up completely orphaned Stubs (nodes with no edges)
    sqlx::query(
        r#"
        DELETE FROM nodes 
        WHERE node_type = 'Stub' 
        AND id NOT IN (SELECT source_id FROM edges) 
        AND id NOT IN (SELECT target_id FROM edges)
        "#,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_dependency_context(
    pool: &SqlitePool,
    node_id: &str,
    max_depth: u32,
    max_tokens: usize,
) -> Result<Vec<String>, sqlx::Error> {
    // Use a Recursive CTE to traverse outgoing (dependencies) and incoming (callers) edges
    let query = r#"
        WITH RECURSIVE traverse(id, depth) AS (
            SELECT ?, 0
            UNION
            -- Outgoing edges (What this node depends on / calls)
            SELECT e.target_id, t.depth + 1
            FROM edges e
            JOIN traverse t ON e.source_id = t.id
            WHERE t.depth < ?
            UNION
            -- Incoming edges (What depends on / calls this node)
            SELECT e.source_id, t.depth + 1
            FROM edges e
            JOIN traverse t ON e.target_id = t.id
            WHERE t.depth < ?
        )
        SELECT n.id, n.node_type, n.name, n.file_path, n.line, n.end_line, n.signature, MIN(t.depth) as min_depth
        FROM nodes n
        JOIN traverse t ON n.id = t.id
        GROUP BY n.id
        ORDER BY min_depth ASC;
    "#;

    let rows = sqlx::query(query)
        .bind(node_id)
        .bind(max_depth)
        .bind(max_depth)
        .fetch_all(pool)
        .await?;

    struct NodeContext {
        node_type: String,
        file_path: Option<String>,
        line: Option<i64>,
        end_line: Option<i64>,
        source_code: Option<String>,
        base_string: String,
    }

    let mut nodes = Vec::new();
    let mut total_estimated_tokens = 0;

    for row in rows {
        let id: String = row.try_get("id")?;
        let node_type: String = row.try_get("node_type")?;
        let file_path: Option<String> = row.try_get("file_path")?;
        let line: Option<i64> = row.try_get("line")?;
        let end_line: Option<i64> = row.try_get("end_line")?;
        let signature: Option<String> = row.try_get("signature")?;

        let base_string = if let Some(sig) = &signature {
            format!("### {}\nSignature: {}", id, sig)
        } else {
            format!("### {}", id)
        };

        let node_tokens = estimate_tokens(&base_string);
        if total_estimated_tokens + node_tokens > max_tokens {
            break; // Stop adding nodes to context if we hit the limit
        }

        total_estimated_tokens += node_tokens;

        nodes.push(NodeContext {
            node_type,
            file_path,
            line,
            end_line,
            source_code: None,
            base_string,
        });
    }

    // Second pass: try to add source code
    for node in &mut nodes {
        if node.node_type == "Function" || node.node_type == "Class" {
            if let (Some(path), Some(start), Some(end)) =
                (&node.file_path, node.line, node.end_line)
            {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let lines: Vec<&str> = content.lines().collect();
                    if (start as usize) < lines.len()
                        && (end as usize) < lines.len()
                        && start <= end
                    {
                        let snippet = lines[start as usize..=end as usize].join("\n");
                        let snippet_string = format!("\n```python\n{}\n```", snippet);
                        let snippet_tokens = estimate_tokens(&snippet_string);

                        if total_estimated_tokens + snippet_tokens <= max_tokens {
                            total_estimated_tokens += snippet_tokens;
                            node.source_code = Some(snippet_string);
                        }
                    }
                }
            }
        }
    }

    let mut context_items = Vec::new();
    for node in nodes {
        if let Some(src) = node.source_code {
            context_items.push(format!("{}{}", node.base_string, src));
        } else {
            context_items.push(format!(
                "{}\n(Source code omitted due to context limits)",
                node.base_string
            ));
        }
    }

    Ok(context_items)
}
