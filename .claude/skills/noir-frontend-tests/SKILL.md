---
name: noir-frontend-tests
description: Guide for writing noirc_frontend unit tests. Use when adding, writing, or reviewing frontend tests — regression tests, reproduction tests, error-checking tests, or should_panic tests in the compiler frontend.
user-invocable: false
---

# Writing noirc_frontend Unit Tests

Tests live under `compiler/noirc_frontend/src/tests/`. The root module `tests.rs` defines helpers and declares submodules organized by topic.

## Helpers in `tests.rs`

Importable from submodules via `use crate::tests::{...}`.

### Compilation

| Helper | Use when |
|--------|----------|
| `assert_no_errors(src)` | Program should compile without errors. Panics with `"Expected no errors"` and prints diagnostics. Returns `Context`. |
| `assert_no_errors_without_report(src)` | Same but doesn't print diagnostics on failure. |
| `get_program_errors(src) -> Vec<CompilationError>` | Get raw error list for manual inspection. |
| `get_program_using_features(src, &[UnstableFeature])` | Compile with specific unstable features enabled. |

### Error Checking with Inline Annotations

| Helper | Use when |
|--------|----------|
| `check_errors(src)` | Program has inline error annotations (see below). |
| `check_errors_with_stdlib(src, stdlib_src)` | Same but prefixes stdlib snippets. |
| `check_errors_using_features(src, &[UnstableFeature])` | Same with unstable features. |
| `check_monomorphization_error(src)` | Error occurs during monomorphization, not elaboration. |

### Inline Error Annotation Syntax

Used with `check_errors()`. Error markers go on the line below the code that produces the error:

```rust
let src = r#"
    impl Foo::Bar { }
         ^^^^^^^^ Cannot define a trait impl on associated types
         ~~~~~~~~ secondary message here

    fn main() { }
"#;
check_errors(src);
```

- `^^^` = primary error span + message
- `~~~` = secondary error span + message
- Markers align character-by-character with the code line above

## Helpers in `test_utils.rs`

Importable via `use crate::test_utils::{...}`. Also usable by other crates via the `test_utils` feature.

| Helper | Use when |
|--------|----------|
| `get_program(src)` | Raw `(ParsedModule, Context, Vec<CompilationError>)`. No stdlib. |
| `get_program_with_options(src, GetProgramOptions)` | Full control over compilation. |
| `get_monomorphized(src)` | Compile + monomorphize. Returns `Result<Program, MonomorphizationError>`. |
| `get_monomorphized_with_stdlib(user_src, stdlib_src)` | Monomorphize with stdlib prefix. |
| `stdlib_src::ZEROED`, `EQ`, `ORD` | Pre-written stdlib snippets for tests needing stdlib traits. |

### `GetProgramOptions`

```rust
GetProgramOptions {
    allow_parser_errors: bool,       // default: false
    allow_elaborator_errors: bool,   // default: false — needed for check_errors
    root_and_stdlib: bool,           // default: false — treat as stdlib (enables builtins)
    frontend_options: FrontendOptions,
}
```

## Patterns

### Regression test for a bug that should compile (already fixed)

```rust
/// Regression test for https://github.com/noir-lang/noir/issues/XXXXX
#[test]
fn descriptive_name() {
    let src = r#"
    // ... Noir program ...
    fn main() {}
    "#;
    assert_no_errors(src);
}
```

### Reproduction test for a bug that should compile (not yet fixed)

```rust
/// TODO(https://github.com/noir-lang/noir/issues/XXXXX): remove should_panic once fixed
#[test]
#[should_panic(expected = "Expected no errors")]
fn descriptive_name() {
    let src = r#"
    // ... Noir program ...
    fn main() {}
    "#;
    assert_no_errors(src);
}
```

### Test for expected compile error

```rust
#[test]
fn descriptive_name() {
    let src = r#"
    fn main() {
        let x: bool = 42;
                       ^^ expected type annotation
    }
    "#;
    check_errors(src);
}
```

## Gotchas

- **Warnings count as errors**: `get_program` returns warnings (dead code, unused functions) in the `Vec<CompilationError>`. Unused items cause `assert_no_errors` to fail.
  - Fix: mark items `pub` to suppress "unused" warnings.
  - `pub struct`, `pub fn` for items not called from `main()`.
  - If a struct is pub but never constructed, that's still a warning — add `pub` to fields too, or construct it.
- **Always include `fn main() {}`**: Programs without `main` will fail differently.
- **No stdlib by default**: `get_program` doesn't include the stdlib. Use `check_errors_with_stdlib` or `root_and_stdlib: true` if you need stdlib types/traits.
- **Run command**: `cargo nextest run -p noirc_frontend -E 'test(test_name)'`
