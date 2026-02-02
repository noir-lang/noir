# Noir Monorepo Development Guide

## Project Overview

Noir is a domain-specific language for SNARK proving systems (zero-knowledge proofs). The compiler is written in Rust and produces ACIR (Abstract Circuit Intermediate Representation), which can be consumed by any ACIR-compatible proving backend. The project also includes a CLI (`nargo`), LSP server, debugger, formatter, and JavaScript/WASM bindings.

## Architecture

### Compilation Pipeline

```
Source Code → [Lexing] → Tokens → [Parsing] → AST → [Name Resolution + Type Checking (Elaboration)] → HIR → [Monomorphization] → Monomorphized AST → [SSA Generation] → SSA → [SSA Optimizations] → ACIR/Brillig
```

### Workspace Structure

**Compiler** (`compiler/`):
- `noirc_frontend` — Lexer, parser, elaborator (name resolution + type checking), monomorphization. Entry point for the frontend pipeline.
- `noirc_evaluator` — SSA generation, SSA optimization passes, ACIR generation, Brillig generation. The middle/back-end.
- `noirc_driver` — Orchestrates the full compilation pipeline from source to artifacts.
- `fm` — File manager abstraction for source file handling.
- `noirc_errors` — Error reporting with source spans.

**ACVM** (`acvm-repo/`):
- `acir` — Circuit intermediate representation (analogous to LLVM IR for circuits).
- `brillig` — Bytecode format for unconstrained (non-deterministic) execution.
- `acvm` — Virtual machine that executes ACIR circuits.
- `brillig_vm` — Virtual machine that executes Brillig bytecode.
- `blackbox_solver`, `bn254_blackbox_solver` — Cryptographic primitives (hash functions, elliptic curve ops).

**Tooling** (`tooling/`):
- `nargo_cli` — Main CLI tool. Also hosts integration test harness (`tests/execute.rs`, `tests/stdlib-tests.rs`).
- `nargo` — Package manager core (dependency resolution, workspace handling).
- `lsp` — Language Server Protocol implementation.
- `nargo_fmt` — Code formatter.
- `noirc_abi` — ABI handling (conversion between JSON/TOML inputs and Noir types).

**Standard Library** (`noir_stdlib/`): Pure Noir implementations of stdlib functions (arrays, hashing, crypto, etc.).

### Test Programs

`test_programs/` contains integration test suites organized by expected outcome:
- `execution_success/` — Programs that should execute successfully (have `Prover.toml` inputs).
- `execution_failure/` — Programs that should fail at runtime.
- `compile_failure/` — Programs that should fail to compile.
- `compile_success_empty/` — Programs that compile to empty circuits.
- `compile_success_contract/` — Smart contract compilation tests.

Test cases are auto-generated from these directories by `tooling/nargo_cli/build.rs`.

### Key Patterns

- **Unsafe code is forbidden** (`#![forbid(unsafe_code)]` workspace-wide).
- **SSA passes** live in `compiler/noirc_evaluator/src/ssa/opt/` — each module has its own unit tests.
- **Elaboration** (`compiler/noirc_frontend/src/elaborator/`) combines name resolution and type checking in a single pass.
- PRs are **squash-merged** into `master`.

## Build & Development Commands

The project uses `just` as a task runner and `cargo` for Rust builds. Minimum Rust version: 1.85.0. Run `just --list` to see all available commands.

### Building

```bash
cargo build                          # Build default members (nargo_cli, acvm_cli, etc.)
cargo build -p noirc_frontend        # Build a specific crate
cargo build --release                # Release build
```

### Testing (Rust / Noir)

```bash
just test                                              # Full test suite (uses cargo nextest)
cargo nextest run --workspace                          # Equivalent to above
cargo nextest run -p noirc_frontend                    # Tests for a specific crate
cargo nextest run -p nargo_cli --test execute           # Integration tests (execution)
cargo nextest run -p nargo_cli --test execute sha256    # Single integration test by name
cargo test -p nargo_cli --test stdlib-tests             # Noir stdlib tests
cargo test -p nargo_cli --test stdlib-tests -- run_stdlib_tests array  # Stdlib tests for one module
cargo test -p noir_ast_fuzzer --test smoke              # Fuzz tests (quick)
```

Integration tests use `insta` for snapshot testing. When adding new tests or changing outputs, review and accept snapshots with `cargo insta review`.

### Testing (JavaScript/TypeScript)

**Never run `yarn test` from the project root — always cd into a specific package first.**

```bash
cd <package-name>
yarn test FILENAME                    # Run test file
yarn test FILENAME -t 'test-name'     # Run specific test
```

Before running JS tests, compile first:

```bash
yarn build                            # Full JS compilation
cd <package-name> && yarn build       # Or a specific package
```

### Formatting & Linting

```bash
# Rust
just format          # cargo fmt --all
just format-check    # Check without applying
just clippy          # cargo clippy (release mode, all targets)

# Noir
just format-noir     # Format Noir source files (stdlib + test programs)

# TypeScript/JavaScript
just lint            # ESLint
```

### Installing Dev Tools

```bash
just install-tools        # All tools (Rust + JS + Foundry)
just install-rust-tools   # nextest, insta, cargo-mutants
```

### Dependency Management

After modifying any `package.json`:

```bash
yarn install
```

## 🔀 Git & PR Guidelines

### Branch Naming

Prefix branches with author initials:

```
ab/feature-name
jd/fix-something
```

**Setting Author Initials:**
Configure your git initials for automatic branch naming:

```bash
# Local repository only
git config user.initials "jd"

# Global (all repositories)
git config --global user.initials "jd"
```

**How Claude Determines Author Initials:**

1. First checks `git config user.initials`
2. If not set, derives from `git config user.name` (e.g., "John Doe" → "jd")
3. Uses lowercase initials for branch names
4. Ask to set the user's initials for them if unset.

### Commit Messages - Conventional Commits

Follow [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)

**Supported types** (from `.github/workflows/pull-request-title.yml`):

- `fix`: Bug fixes
- `feat`: New features
- `chore`: Maintenance tasks

**Format**:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Branch Strategy

- **Primary development**: `master` branch (default PR target)
- **Default PR target**: `master` (unless specified otherwise)

### Breaking Changes

When introducing breaking changes:

1. **Include in PR description**: Clearly document the breaking changes

### CI Labels

Special labels to control CI behavior:

- **`show-bench`**: Print the comparisons of CI benchmarks between the PR commit and the base branch.

  - Use when you think that the PR will result in improvements/degradation of compilation time or memory usage.

### Workflow Reminders

- ✅ Always compile before testing
- ✅ Format/lint modified packages before committing
- ✅ Run tests for modified code
- ✅ Use single-package commands when possible (faster)
- ❌ Never run `yarn test` from project root - always cd into package first

## Workflow Orchestration

### Plan Mode
- Enter plan mode for any non-trivial task (3+ steps, multi-file changes, or architectural decisions).
- Include verification steps in the plan upfront, not as an afterthought.
- If new information invalidates the plan: stop, update the plan, then continue.
- Write a crisp spec first when requirements are ambiguous (inputs/outputs, edge cases, success criteria).

### Subagent Strategy
- Use subagents to keep the main context clean and to parallelize: repo exploration, pattern discovery, test failure triage, dependency research.
- Give each subagent one focused objective and a concrete deliverable.
- Merge subagent outputs into a short, actionable synthesis before coding.

### Incremental Delivery
- Prefer thin vertical slices over big-bang changes.
- Land work in small, verifiable increments: implement, test, verify, then expand.
- When feasible, keep changes behind safe defaults or conditional checks.

### Verification Before "Done"
- Never mark a task complete without evidence: tests pass, lint/typecheck clean, build succeeds.
- Compare baseline vs changed behavior when relevant.

### Autonomous Bug Fixing
- When given a bug report: reproduce, isolate root cause, fix, add regression coverage, verify.
- Do not offload debugging work to the user unless truly blocked.
- If blocked, ask for one specific missing detail with a recommended default.

### Lessons Learned
- After any user correction or discovered mistake, add an entry to `tasks/lessons.md` capturing:
  - The failure mode, the detection signal, and a prevention rule.
- Review `tasks/lessons.md` at session start and before major refactors.
- This file acts as an evolving extension of CLAUDE.md for project-specific learnings.
