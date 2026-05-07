//! Compile-fail tests for `MsgpackTagged`.
//!
//! Each `tests/compile_fail/*.rs` file is a small program that should *fail* to
//! compile, and the corresponding `.stderr` file captures the expected error
//! message. `trybuild` runs the compiler on each file, asserts the build fails,
//! and diffs the actual error output against the captured `.stderr`.
//!
//! Adding a new case:
//! 1. Drop a new `.rs` file under `tests/compile_fail/`.
//! 2. Run `TRYBUILD=overwrite cargo test -p msgpack_tagged --test compile_fail`
//!    to capture the actual compiler output into a sibling `.stderr` file.
//! 3. Review the `.stderr` file (it's the contract — what user-visible error we
//!    expect) and check it in.

#[test]
fn compile_fail() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/compile_fail/*.rs");
}
