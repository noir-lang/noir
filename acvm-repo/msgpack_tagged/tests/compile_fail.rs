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

/// Trybuild internally spawns its own `cargo build` against a freshly-generated
/// workspace that has a path-dep on `msgpack_tagged_derive`. That inner build
/// has to resolve `proc-macro2` / `quote` / `syn` from crates.io, so it needs
/// the cargo registry to be reachable.
///
/// CI's run-tests job runs from a `cargo nextest archive` artifact, with no
/// registry cache restored — which fails the inner trybuild build. That job
/// sets `NOIR_NEXTEST_ARCHIVED=1` (see `.github/workflows/test-rust-workspace*.yml`)
/// to opt this test out. CI exercises the test instead in the
/// build-test-artifacts job, where the registry cache is intact, via plain
/// `cargo test` (env var unset).
#[test]
fn compile_fail() {
    if std::env::var_os("NOIR_NEXTEST_ARCHIVED").is_some() {
        eprintln!(
            "skipping trybuild compile_fail under nextest archive — needs cargo registry; \
             CI runs it separately from the build job",
        );
        return;
    }
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/compile_fail/*.rs");
}
