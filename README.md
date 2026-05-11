# Code Graph Tool

A local-first CLI tool that creates Knowledge Graphs from codebases using GraphRAG (Retrieval-Augmented Generation) principles. Built in Rust for performance, security, and easy deployment.

## 🌟 Features

- **📊 Multi-language Support** - Parse Python, Rust, JavaScript/TypeScript, Go, C/C++
- **🔗 Dependency Tracking** - Track imports, function calls, class inheritance
- **💾 Local Storage** - SQLite database with incremental updates (checksum-based)
- **🤖 LLM Integration** - Inject relevant structured context into prompts
- **🔌 Dual API Support** - MCP Server (Claude/Cursor) + REST API
- **⚡ Fast & Efficient** - Async runtime, transaction-based DB operations
- **🔒 Privacy-First** - Local-first design, no data leaves your machine

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/code-graph-tool.git
cd code-graph-tool

# Build the project
cargo build --release
```

### Basic Usage

#### Parse a Codebase and Query

```bash
# Parse and get LLM analysis of a specific function
cargo run --release -- \
  --path ./src \
  --target "main.rs:main" \
  --prompt "Explain what this function does"
```

#### Discovery Mode (Find Functions by Name)

```bash
# Search for functions matching a pattern
cargo run --release -- \
  --discover-name "calculate_sum" \
  --search-only

# Use discovery with LLM analysis
cargo run --release -- \
  --discover-name "process_data" \
  --prompt "Analyze this function's error handling"
```

#### Configure Context Retrieval

```bash
# Customize dependency depth and token limits
cargo run --release -- \
  --path ./src \
  --target "parser.rs:parse_file" \
  --depth 3 \
  --max-tokens 5000 \
  --prompt "What are the dependencies of this function?"
```

## 🔧 Environment Variables

Configure LLM integration via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENAI_BASE_URL` | `https://api.openai.com/v1` | LLM API endpoint |
| `OPENAI_API_KEY` | (required) | API authentication key |
| `OPENAI_MODEL` | `gpt-4o-mini` | Model to use for analysis |

### Example Setup

```bash
export OPENAI_BASE_URL="https://api.openai.com/v1"
export OPENAI_API_KEY="your-api-key-here"
export OPENAI_MODEL="gpt-4o-mini"
```

## 📡 Server Modes

### MCP Server (Recommended for Claude/Cursor)

Expose the tool as an MCP server for integration with AI clients:

```bash
cargo run --release --bin mcp-server
```

**Supported Clients:**
- Claude Desktop
- Cursor IDE
- Continue.dev
- Any MCP-compatible client

#### Configure for Claude Desktop

Create `~/.config/claude/mcp.json`:

```json
{
  "mcpServers": {
    "code-graph": {
      "command": "cargo",
      "args": ["run", "--release", "--bin", "mcp-server"],
      "env": {}
    }
  }
}
```

### REST API Server

Start a RESTful API server for custom integrations:

```bash
cargo run --release --bin api-server
```

**Available Endpoints:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/search` | POST | Search for nodes by name pattern |
| `/context` | POST | Get detailed context for a node |
| `/dependencies` | POST | Get full dependency graph |
| `/files` | POST | List all parsed files |

#### API Example (cURL)

```bash
# Search for functions
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{"pattern": "main", "node_type": "Function"}'

# Get node context
curl -X POST http://localhost:3000/context \
  -H "Content-Type: application/json" \
  -d '{"node_id": "src/main.rs:main", "max_depth": 2}'
```

## 📚 CLI Reference

```
Usage: code-graph-tool [OPTIONS] --path <PATH> [--target <TARGET>]

Options:
  -p, --path <PATH>
          The path to a file or directory to parse [default: src]

  -t, --target <TARGET>
          The target node to fetch context for (e.g., "src/sample.py:main")

      --discover-name <NAME>
          Function or class name to discover (requires --discover-name)

      --prompt <PROMPT>
          The prompt to send to the LLM [default: ""]

      --search-only
          Only search for nodes, don't call LLM

  -d, --depth <DEPTH>
          Maximum depth for dependency context retrieval [default: 2]

  -m, --max-tokens <TOKENS>
          Maximum token limit for context generation [default: 2000]

  -h, --help
          Print help

  -V, --version
          Print version
```

## 🏗️ Architecture

### Data Model

**Nodes:**
- `File` - Source code files with metadata (path, language, size, checksum)
- `Function` - Functions/methods with signatures and line numbers
- `Class` - Class definitions with inheritance information
- `Stub` - Forward references for unresolved dependencies

**Edges:**
- `Contains` - File contains function/class
- `Calls` - Function calls another function
- `Imports` - File imports a module
- `InheritsFrom` - Class inherits from parent class

### Tech Stack

- **Language:** Rust 2021 Edition
- **Async Runtime:** Tokio
- **CLI Framework:** Clap
- **Code Parsing:** Tree-sitter (multi-language)
- **Graph Library:** Petgraph
- **Database:** SQLx + SQLite
- **HTTP Client:** Reqwest
- **API Server:** Axum

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_full_parsing_and_querying
```

## 🔒 Security

This tool is designed with security in mind:

- **Local-first:** All data stays on your machine
- **No hardcoded secrets:** API keys from environment variables only
- **Parameterized queries:** SQL injection prevention
- **Input validation:** Whitelist-based file extension filtering
- **Safe file access:** Paths controlled by internal graph, not direct user input

## 📝 Supported Languages

| Language | Extensions | Status |
|----------|-----------|--------|
| Python | `.py` | ✅ Full |
| Rust | `.rs` | ✅ Full |
| JavaScript | `.js` | ✅ Full |
| TypeScript | `.ts` | ✅ Full |
| Go | `.go` | ✅ Full |
| C++ | `.cpp`, `.cc`, `.cxx` | ✅ Full |
| C | `.c`, `.h` | ✅ Full |

## 🛠️ Development

### Build from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run all targets
cargo build --all-targets
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check
```

### Clean Build

```bash
# Clean build artifacts
cargo clean

# Remove database (if schema changes)
rm graphrag.db
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Tree-sitter for robust multi-language parsing
- Petgraph for graph data structures
- SQLx for async database operations
- The Rust and Open Source community

## 📞 Support

For issues, questions, or feature requests, please open an issue on GitHub.

---

**Built with ❤️ in Rust**
