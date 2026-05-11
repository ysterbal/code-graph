---
name: AGENTS
description: Authoritative guide for AI agents working on code-graph-tool (GraphRAG) project
version: 2.0
tags:
  - rust
  - cli
  - graph-rag
  - llm-integration
  - tree-sitter
alwaysApply: true
---

# AGENTS.md

> **Version:** 2.0  
> **Last Updated:** May 10, 2026  

This file contains authoritative instructions and context for AI agents assisting with the `code-graph-tool` (GraphRAG) project.

## 🎯 Project Overview

A **local-first CLI tool** built in Rust that creates Knowledge Graphs from codebases using GraphRAG (Retrieval-Augmented Generation) principles. The tool parses source code, builds dependency graphs, stores them in SQLite, and injects relevant structured context into LLM prompts to reduce token usage and improve code analysis accuracy.

### Key Capabilities

- **Multi-language parsing**: Python, Rust, JavaScript/TypeScript, Go, C/C++
- **Dependency tracking**: Imports, function calls, class inheritance
- **Local-first storage**: SQLite with checksum-based incremental updates
- **LLM integration**: Context-aware prompts with token budgeting
- **Dual API support**: MCP Server (Claude/Cursor) + REST API
- **Structured logging**: `tracing` for observability

## 🛠️ Tech Stack

| Category | Technology | Purpose |
| ---------- | ----------- | --------- |
| **Language** | Rust 2021 Edition | Performance, memory safety |
| **Async Runtime** | `tokio` | Async/await concurrency |
| **CLI Framework** | `clap` (v4) | Command-line parsing |
| **Code Parsing** | `tree-sitter` + language bindings | AST extraction |
| **Graph Library** | `petgraph` (v0.6) | Graph data structures |
| **Database** | `sqlx` + SQLite | Local persistence |
| **HTTP Client** | `reqwest` | LLM API calls |
| **API Server** | `axum` (v0.7) | REST endpoints |
| **Logging** | `tracing` + `tracing-subscriber` | Structured logging |
| **Config** | `toml` + `serde` | Configuration management |

## 🚀 Quick Start Commands

### Building & Testing

```bash
# Build all targets (main + binaries)
cargo build --all-targets

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Lint with strict warnings
cargo clippy -- -D warnings

# Check formatting compliance
cargo fmt --check
```

### Running the Tool

```bash
# Parse and query a specific function
cargo run -- --path ./src --target "main.rs:main" --prompt "Explain this function"

# Discovery mode - find functions by name
cargo run -- --discover-name "calculate_sum" --search-only

# Use custom config file
cargo run -- --config .graphrag.toml --path ./src --target "parser.rs:parse_file"

# Verbose debugging
echo RUST_LOG=code_graph_tool=debug && cargo run -- ...  
```

### Server Modes

```bash
# Start MCP server (for Claude/Cursor)
cargo run --bin mcp-server

# Start REST API server
cargo run --bin api-server
```

### Maintenance

```bash
# Clean build artifacts
cargo clean

# Remove database (when schema changes)
rm graphrag.db

# Check dependencies
cargo tree
```

## 📁 Project Structure

```text
code-graph-tool/
├── README.md                    # User-facing documentation ⭐
├── AGENTS.md                    # This file - agent guidelines
├── INTEGRATION_GUIDE.md         # API integration examples
├── config.example.toml          # Configuration template
├── Cargo.toml                   # Dependencies & binary targets
│
├── src/
│   ├── main.rs                  # CLI entry point + tracing setup
│   ├── lib.rs                   # Module exports
│   ├── config.rs                # Configuration management (NEW)
│   ├── db.rs                    # Database operations + logging
│   ├── graph.rs                 # Petgraph wrapper + unit tests ⭐
│   ├── models.rs                # Data structures (File, Function, Class)
│   ├── parser.rs                # Tree-sitter AST extraction
│   ├── llm.rs                   # LLM API client
│   └── tests/
│       └── integration.rs       # Integration tests
│
├── src/tools/
│   ├── mcp-server.rs            # MCP protocol server
│   └── api-server.rs            # REST API server (Axum)
│
└── target/                      # Build artifacts (gitignored)
```

### Key Files to Know

- **`src/main.rs`**: Entry point, CLI argument parsing, tracing initialization
- **`src/config.rs`**: Configuration loading from TOML/env vars
- **`src/db.rs`**: Database operations with structured logging
- **`src/graph.rs`**: Graph wrapper with comprehensive unit tests
- **`src/parser.rs`**: Multi-language AST extraction using tree-sitter
- **`src/llm.rs`**: OpenAI-compatible API client
- **`src/tools/mcp-server.rs`**: JSON-RPC MCP server
- **`src/tools/api-server.rs`**: REST API with Axum router

## 🎨 Code Style & Architecture Guidelines

### Rust Best Practices

```rust
// ✅ DO: Use Result and ? for error handling
fn parse_file(path: &str) -> Result<File, ParseError> {
    let content = std::fs::read_to_string(path)?;
    Ok(File::new(content))
}

// ❌ DON'T: Avoid unwrap() in production code
let content = std::fs::read_to_string(path).unwrap();
```

### Error Handling Patterns

```rust
// Use descriptive error types when possible
#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

// Propagate with context using ?
let pool = db::init_db(&config.db_url()).await?;
```

### Memory & Performance

```rust
// ✅ DO: Use static strings where possible
impl EdgeType {
    pub fn as_str(&self) -> &'static str { // No allocation!
        match self {
            EdgeType::Contains => "Contains",
            // ...
        }
    }
}

// ✅ DO: Use transactions for bulk operations
let mut tx = pool.begin().await?;
// ... multiple inserts
tx.commit().await?;
```

### Async Patterns

```rust
// ✅ DO: Use tokio::main for async entry points
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Async operations
}

// ✅ DO: Wrap blocking I/O appropriately
let content = tokio::task::spawn_blocking(|| {
    std::fs::read_to_string(path)
}).await?;
```

### Logging with Tracing

```rust
// ✅ DO: Use tracing macros instead of println!
use tracing::{info, debug, warn, error};

info!("Initializing database at {}", db_url);
debug!("Parsing file: {:?}", path);
warn!("File not found, skipping");
error!("Failed to parse: {}", e);
```

### Testing Guidelines

```rust
// ✅ DO: Write comprehensive unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_new() {
        let graph = CodeGraph::new();
        assert_eq!(graph.nodes.node_count(), 0);
    }

    #[test]
    async fn test_database_operations() {
        let pool = create_test_db().await;
        // Test database operations
    }
}
```

### Configuration Management

```rust
// ✅ DO: Use Config struct for all settings
let config = Config::from_env(); // or from_file()

// Environment variable overrides
pub fn base_url(&self) -> String {
    std::env::var("OPENAI_BASE_URL")
        .unwrap_or(self.base_url.clone())
}
```

## 🏗️ Architecture Patterns

### Graph Data Model

**Nodes:**

```rust
pub enum NodeData {
    File(FileNode),      // Source files with metadata
    Function(FunctionNode), // Functions/methods
    Class(ClassNode),     // Class definitions
    Stub(String),         // Forward references
}
```

**Edges:**

```rust
pub enum EdgeType {
    Contains,        // File contains function/class
    Calls,           // Function calls another
    Imports,         // File imports module
    InheritsFrom,    // Class inherits from parent
}
```

**Node ID Format:**

- Files: `file_path` (e.g., `src/main.rs`)
- Functions: `file_path:function_name` (e.g., `src/main.rs:main`)
- Classes: `file_path:class_name` (e.g., `src/models.py:User`)

### Database Schema

```sql
-- Nodes table
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,  -- File, Function, Class, Stub
    name TEXT,
    file_path TEXT,
    language TEXT,
    size INTEGER,
    line INTEGER,
    end_line INTEGER,
    signature TEXT,
    checksum TEXT
);

-- Edges table
CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relation TEXT NOT NULL,
    PRIMARY KEY (source_id, target_id, relation),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
);
```

### Incremental Update Strategy

1. **Checksum-based detection**: Compare file checksums
2. **Targeted pruning**: Remove only changed files' edges
3. **Stub preservation**: Downgrade to Stub instead of delete
4. **Orphan cleanup**: Remove nodes with no edges

```rust
// Example: Handle file change
if new_checksum != old_checksum {
    db::clear_file_state(&pool, &file_path).await?;
    parser::parse_file(&file_path, &code, &checksum, &mut graph)?;
}
```

### Token Budgeting

```rust
// Context retrieval respects token limits
let context = db::get_dependency_context(
    &pool,
    node_id,
    max_depth: 2,
    max_tokens: 2000  // Hard limit
).await?;
```

## 📊 Code Structure

### Core Modules (`src/`)

| Module | Purpose | Key Functions |
| -------- | --------- | --------------- |
| `main.rs` | CLI entry point | Argument parsing, tracing setup |
| `lib.rs` | Module exports | Re-exports common types |
| `config.rs` | Configuration | `Config::from_env()`, `from_file()` |
| `db.rs` | Database ops | `init_db()`, `save_graph()`, `get_context()` |
| `graph.rs` | Graph wrapper | `add_function()`, `add_call_edge()` |
| `models.rs` | Data structures | `FileNode`, `FunctionNode`, `EdgeType` |
| `parser.rs` | AST extraction | `parse_file()`, `traverse_and_extract()` |
| `llm.rs` | LLM client | `send_to_llm()` |
| `tests/` | Integration tests | Full workflow validation |

### Tool Binaries (`src/tools/`)

| Binary          | Purpose            | Port | Protocol   |
| --------------- | ------------------ | ---- | ---------- |
| `mcp-server.rs` | MCP protocol server | -    | JSON-RPC   |
| `api-server.rs` | REST API server    | 3000 | HTTP/JSON  |

### Configuration

**Environment Variables:**

```bash
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_API_KEY=your-key-here
OPENAI_MODEL=gpt-4o-mini
GRAPH_RAG_DB_PATH=graphrag.db
GRAPH_RAG_MAX_DEPTH=2
GRAPH_RAG_MAX_TOKENS=2000
RUST_LOG=info  # or debug, warn, error
```

**TOML Config (`config.example.toml`):**

```toml
[database]
path = "graphrag.db"
max_connections = 5

[llm]
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"

[parser]
max_depth = 2
max_tokens = 2000
```

## 🔒 Security Guidelines

### API Key Management

```rust
// ✅ DO: Use environment variables
let api_key = std::env::var("OPENAI_API_KEY")
    .unwrap_or_default();

// ❌ DON'T: Hardcode secrets
let api_key = "sk-..."; // NEVER!
```

### SQL Injection Prevention

```rust
// ✅ DO: Use parameterized queries
sqlx::query("SELECT * FROM nodes WHERE id = ?")
    .bind(node_id)
    .fetch_all(&pool)
    .await?;

// ❌ DON'T: String concatenation
let query = format!("SELECT * FROM nodes WHERE id = '{}'", node_id);
```

### Input Validation

```rust
// ✅ DO: Whitelist file extensions
let valid_exts = ["py", "rs", "js", "ts", "go", "cpp", "c", "h"];
if valid_exts.contains(&ext) {
    // Process file
}
```

## 🧪 Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_type_as_str() {
        assert_eq!(EdgeType::Contains.as_str(), "Contains");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_parsing_and_querying() {
    let pool = create_test_db().await;
    
    // Setup: Parse code
    let mut graph = CodeGraph::new();
    parser::parse_code(code, "test.py", "checksum", ...)?;
    db::save_graph(&graph, &pool).await?;
    
    // Execute: Query context
    let context = db::get_dependency_context(&pool, "test.py:main", 2, 5000).await?;
    
    // Verify: Check results
    assert!(!context.is_empty());
    assert!(context[0].contains("main"));
}
```

### Test Database Helper
```rust
async fn create_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    // Initialize schema
    sqlx::query("CREATE TABLE nodes (...)")
        .execute(&pool)
        .await
        .unwrap();
    pool
}
```

## 📝 Documentation Standards

### Public API Documentation

```rust
/// Estimate token count using a simple heuristic
/// 
/// For more accurate counting, use tiktoken_rs with proper encoding
fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}
```

### README Requirements

- Quick start guide
- Feature list
- Environment variables
- Usage examples
- API documentation

## 🚀 Deployment Checklist

- [ ] All tests passing (`cargo test`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] README.md updated with new features
- [ ] Configuration template provided
- [ ] Environment variables documented
- [ ] API endpoints tested
- [ ] Error messages reviewed

## 🐛 Common Issues & Solutions

### Issue: "Database schema changed"

**Solution:** `rm graphrag.db && cargo run -- ...`

### Issue: "No nodes found"

**Solution:** Ensure codebase is parsed first with `cargo run -- --path ./src`

### Issue: "API key required"

**Solution:** Set `OPENAI_API_KEY` environment variable

### Issue: "Parse error for file"

**Solution:** Check file syntax or language support in `parser.rs`

## 📚 Related Documentation

- **README.md**: User-facing documentation
- **INTEGRATION_GUIDE.md**: API integration examples
- **config.example.toml**: Configuration template
- **src/config.rs**: Configuration module implementation
- **src/tests.rs**: Integration test examples

## 🤝 Contributing Guidelines

1. **Fork and branch**: `git checkout -b feature/amazing-feature`
2. **Write tests**: All new features need tests
3. **Format code**: `cargo fmt`
4. **Lint code**: `cargo clippy -- -D warnings`
5. **Update docs**: README and inline documentation
6. **Commit clearly**: Use descriptive commit messages
7. **Open PR**: Describe changes and testing done

## 📞 Support & Resources

- **Issues**: Open GitHub issue for bugs/feature requests
- **Documentation**: Check README.md and INTEGRATION_GUIDE.md
- **Examples**: See `config.example.toml` for config patterns
- **Tests**: Review `src/tests.rs` for usage examples

---

## 📋 Quick Reference Card

### Essential Commands

```bash
# Development
cargo build --all-targets    # Build everything
cargo test                   # Run all tests
cargo fmt                    # Format code
cargo clippy                 # Lint code

# Running the tool
cargo run -- --path ./src --target "main.rs:main" --prompt "Explain"
cargo run --bin mcp-server   # Start MCP server
cargo run --bin api-server   # Start REST API

# Maintenance
cargo clean                  # Clean build artifacts
rm graphrag.db              # Reset database
```

### Environment Setup

```bash
# Required
export OPENAI_API_KEY="your-key-here"

# Optional (with defaults)
export OPENAI_BASE_URL="https://api.openai.com/v1"
export OPENAI_MODEL="gpt-4o-mini"
export RUST_LOG=info
```

### File Checklist for Changes

- [ ] Update code
- [ ] Add/update tests
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy`
- [ ] Run `cargo test`
- [ ] Update documentation
- [ ] Verify builds cleanly

---

#### Last updated: May 10, 2026 | Version 2.0
