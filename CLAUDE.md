# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sensei is a Domain Specific Language for EVM smart contract development, forked from the Noir programming language. The current focus is on developing the language frontend and tooling infrastructure, while removing ZK/SNARK-specific components. No backend code generation is planned at this time.

## Current Architecture

### Components to Keep and Develop
- **Frontend** (`compiler/noirc_frontend/`): Lexer, parser, type checker, semantic analysis
- **Build System** (`tooling/nargo/`, `tooling/nargo_cli/`): Project management and build tools
- **Standard Library** (`sensei_stdlib/`): Custom stdlib for language features
- **LSP** (`tooling/lsp/`): Language Server Protocol for editor support
- **Formatter** (`tooling/nargo_fmt/`): Code formatting
- **Error Handling** (`compiler/noirc_errors/`): Diagnostic and error reporting
- **File Management** (`compiler/fm/`): Source file handling

### Components Being Removed (ZK/Aztec-specific)
- **ACVM** (`acvm-repo/`): Abstract Circuit Virtual Machine
- **SSA/Circuit Generation** (`compiler/noirc_evaluator/`): SNARK circuit compilation
- **Brillig VM** (`acvm-repo/brillig*`): Unconstrained computation VM
- **All proof system components**

### Custom Standard Library (`sensei_stdlib/`)
Active development of EVM-focused language features:
- Collections: `Vec`, `Map`, `BoundedVec` (`collections/`)
- Meta-programming: Compile-time utilities (`meta/`)
- Core language primitives (`hash.nr`, `cmp.nr`, `ops/`)
- Array/slice operations (`array/`, `slice.nr`)

## Common Development Commands

### Building
```bash
# Build Rust components
cargo build

# Build specific frontend components
cargo build -p noirc_frontend
cargo build -p nargo_cli
cargo build -p noir_lsp
```

### Testing
```bash
# Run frontend tests (main focus area)
cargo test -p noirc_frontend

# Run standard library tests
cargo test -p nargo_cli --test stdlib-tests

# Test specific stdlib modules
cargo test -p nargo_cli --test stdlib-tests -- run_stdlib_tests <module_name>

# Run formatter tests
cargo test -p nargo_fmt

# Run LSP tests
cargo test -p noir_lsp
```

### Development Tools
```bash
# Check Sensei program syntax/types (no compilation backend)
cargo run -p nargo_cli -- check

# Format Sensei code
cargo run -p nargo_cli -- fmt

# Start LSP server for editor integration
cargo run -p noir_lsp
```

### Linting
```bash
# Format Rust code
cargo fmt

# Check Rust code
cargo clippy
```

## File Structure

### Project Files
- `Nargo.toml`: Sensei project configuration
- `.nr` files: Sensei source code
- `src/main.nr`: Main entry point for Sensei programs

### Core Development Areas
- `compiler/noirc_frontend/`: **Primary focus** - language implementation
- `tooling/nargo_cli/`: CLI interface and commands
- `tooling/lsp/`: Language server for editor support
- `sensei_stdlib/`: **Active development** - standard library
- `test_programs/`: Integration test programs

## Testing Structure

- **Frontend Tests**: Unit tests in `compiler/noirc_frontend/src/tests/`
- **Language Feature Tests**: Programs in `test_programs/`:
  - `compile_success_*/`: Programs that should parse/typecheck successfully
  - `compile_failure/`: Programs that should fail during frontend processing
- **Stdlib Tests**: Generated from `#[test]` functions in `sensei_stdlib/`

## Current Development Focus

### Language Pipeline (Frontend Only)
1. **Lexing/Parsing** (`noirc_frontend/lexer`, `noirc_frontend/parser`)
2. **Type Checking** (`noirc_frontend/elaborator`)
3. **HIR Generation** (`noirc_frontend/hir`)
4. **Semantic Analysis** (`noirc_frontend/`)

### Current Branch: `chore/nuke-zk-backend`
Working on removing ZK-specific components while preserving:
- Robust language frontend
- Developer tooling (CLI, LSP, formatter)
- Standard library infrastructure

### Active Development Areas
- Language syntax and semantics in `noirc_frontend/`
- Standard library features in `sensei_stdlib/`
- Developer experience improvements in `tooling/`
- Error messages and diagnostics

### Areas to Avoid
- `acvm-repo/`: Being removed
- `compiler/noirc_evaluator/`: ZK compilation backend, being removed
- Any proof/circuit generation code

## Error Handling

Sensei maintains sophisticated error reporting:
- Source position tracking (`noirc_errors/src/position.rs`)
- Call stack information (`noirc_errors/src/call_stack.rs`)
- Detailed diagnostic messages (`noirc_errors/src/reporter.rs`)

## Development Principles

1. **The compiler is mission critical code** - Its logic needs to be sound and work on all user inputs
2. **When in doubt, opt for more readable, easy to reason about implementation** - Clarity over cleverness
3. **You are a senior compiler engineer** - It is not acceptable to hardcode inputs or use brittle workarounds
4. **Use `cargo check` when verifying whether the code compiles** - Faster feedback during development
5. **Tools**: Look under `.claude-tools/` - You will find prepared scripts for common actions; these are usually just aliases - prefer these over running commands directly

## Minimum Requirements

- Rust 1.85.0 or later
- Standard Rust development tools (cargo, rustfmt, clippy)