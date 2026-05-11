# Test Samples

This directory contains sample code files in various programming languages for testing the GraphRAG parser.

## Supported Languages

The parser currently supports **8 programming languages**:

| Language | File | Parser Support |
|----------|------|----------------|
| ✅ Python | `sample.py` | Full support - classes, methods, functions |
| ✅ Rust | `sample.rs` | Full support - structs, impl blocks, methods |
| ✅ JavaScript | `sample.js` | Full support - classes, methods, functions |
| ✅ TypeScript | `sample.ts` | Full support - classes, interfaces, methods |
| ✅ Go | `sample.go` | Full support - structs, functions, methods |
| ✅ Java | `sample.py` | Full support - classes, methods, constructors |
| ⚠️ C | `sample.c` | Limited support - basic function extraction |
| ⚠️ C++ | `sample.cpp` | Limited support - class detection |

## Sample Descriptions

### sample.py (Python)
A Python module demonstrating:
- Class definitions with methods
- Function definitions
- Import statements
- Type hints and annotations

**Nodes extracted:** File, Class, Methods, Functions, Import stubs

### sample.rs (Rust)
A Rust module demonstrating:
- Struct definitions with `pub` fields
- `impl` blocks with methods
- Trait-like patterns
- Generic collections (HashMap)
- Unit tests

**Nodes extracted:** File, Structs, Methods, Functions, Module imports

### sample.js (JavaScript)
A JavaScript module demonstrating:
- ES6 class definitions
- Constructor methods
- Prototype methods
- Inheritance with `extends`
- Event emitter pattern
- CommonJS exports

**Nodes extracted:** File, Classes, Methods, Constructors, Functions

### sample.ts (TypeScript)
A TypeScript module demonstrating:
- Class definitions with type annotations
- Interface implementations
- Generic types (Map, Array)
- Public/private modifiers
- Return type annotations
- ES6 module exports

**Nodes extracted:** File, Classes, Interfaces, Methods, Functions

### sample.go (Go)
A Go module demonstrating:
- Struct definitions
- Pointer receivers for methods
- Package imports
- Map data structures
- Synchronization primitives (sync.Mutex)
- Factory functions

**Nodes extracted:** File, Structs, Functions, Methods

### sample.java (Java)
A Java module demonstrating:
- Class definitions with access modifiers
- Instance fields and constructors
- Method overloading
- Generic types (List, ArrayList, Optional)
- Javadoc comments
- Import statements

**Nodes extracted:** File, Classes, Methods, Constructors, Import stubs

### sample.c (C)
A C module demonstrating:
- Struct typedef definitions
- Function declarations and definitions
- Pointer operations
- Dynamic memory allocation
- String manipulation
- Error handling patterns

**Nodes extracted:** File, Functions (limited struct support)

### sample.cpp (C++)
A C++ module demonstrating:
- Class definitions with access specifiers
- Constructor initialization lists
- Smart pointers (`std::unique_ptr`)
- STL containers (`std::vector`, `std::map`)
- Const methods
- Move semantics

**Nodes extracted:** File, Classes (limited method support)

## Running Tests

To test parsing for a specific language:

```bash
# Parse a single file
cargo run --bin code-graph-tool -- --path ./test_samples/sample.py

# Query specific nodes
cargo run --bin code-graph-tool -- --target "sample.py:CodeParser"

# Discover functions by name
cargo run --bin code-graph-tool -- --discover-name "add_user" --search-only
```

## Unit Tests

All language parsers are tested in `src/parser.rs`:

```bash
# Run all parser tests
cargo test --lib parser::tests

# Run specific language test
cargo test test_extracts_java_classes_and_methods
```

### Test Coverage

| Test | Language | What it Validates |
|------|----------|-------------------|
| `test_extracts_functions` | Python | Classes, methods, imports |
| `test_extracts_rust_structs_and_methods` | Rust | Structs, impl blocks, methods |
| `test_extracts_javascript_classes_and_methods` | JavaScript | ES6 classes, constructors, methods |
| `test_extracts_typescript_classes_and_methods` | TypeScript | Classes with types, interfaces |
| `test_extracts_go_structs_and_methods` | Go | Structs, functions, pointer receivers |
| `test_extracts_java_classes_and_methods` | Java | Classes, constructors, methods |
| `test_extracts_c_functions_and_structs` | C | Basic function extraction |
| `test_extracts_cpp_classes_and_methods` | C++ | Class detection |

## Parser Limitations

### Full Support ✅
- **Python**: Complete AST extraction for all constructs
- **Rust**: Full support for structs, impl blocks, and generics
- **JavaScript**: ES6+ features including classes and arrow functions
- **TypeScript**: All TypeScript-specific features with type annotations
- **Go**: Structs, methods, and standard library patterns
- **Java**: Complete class structure with inheritance

### Limited Support ⚠️
- **C**: Basic function extraction; struct field parsing is limited
- **C++**: Class detection works well, but method extraction within classes has limitations

These limitations are due to the complexity of C/C++ syntax and the tree-sitter grammar implementations.

## Adding New Languages

To add support for a new language:

1. Add the tree-sitter dependency to `Cargo.toml`
2. Import the language in `src/parser.rs`
3. Add the file extension to the match statement in `parse_file()`
4. Create test sample files
5. Add unit tests in `src/parser.rs`

See the Java implementation as a reference for adding new language support.
