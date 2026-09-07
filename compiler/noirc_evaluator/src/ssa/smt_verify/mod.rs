//! Check that a compiler translation preserves behavior for *all* inputs —
//! not just the one concrete input a `Prover.toml`-style execution test
//! would exercise — using cvc5's finite-field SMT theory. [`ssa`] checks
//! that an SSA transformation (a real pass, or a `simplify` rule) preserves
//! behavior; [`acir`] checks SSA-to-ACIR codegen the same way.
//!
//! This shells out to the `cvc5` binary with a hand-written SMT-LIB2 script
//! rather than linking an SMT solver into the compiler: cvc5's finite-field
//! theory (needed to model Noir's prime-field `Field` type) isn't exposed by
//! the safe Rust bindings yet, only by the low-level FFI crate. Tests here are
//! skipped if `cvc5` isn't on `PATH`, so they never affect the default
//! `cargo test` run (except when running on CI).

use std::io::Write as _;
use std::process::{Command, Stdio};

use acvm::AcirField;
use acvm::FieldElement;
use num_bigint::BigUint;

mod acir;
mod ssa;

/// SMT-LIB2 decimal representation of a field element (non-negative, unlike
/// `FieldElement`'s own `Display`, which prints small negative representations).
fn field_to_decimal(value: FieldElement) -> String {
    BigUint::from_bytes_be(&value.to_be_bytes()).to_string()
}

fn field_modulus_decimal() -> String {
    FieldElement::modulus().to_string()
}

/// Whether tests are running in CI, per this repo's own convention
/// (`justfile`'s `ci :=` line checks the same variable the same way).
fn is_ci() -> bool {
    matches!(std::env::var("CI").as_deref(), Ok("true") | Ok("1"))
}

/// Runs `cvc5` on a full SMT-LIB2 script and returns whether it reported
/// `unsat` (`Some(true)`), `sat` (`Some(false)`), or `None` if `cvc5` isn't
/// installed. A missing `cvc5` is only a silent skip locally — in CI it's a
/// hard failure, since a misconfigured install step would otherwise leave CI
/// green having never actually checked anything.
fn run_cvc5(script: &str) -> Option<bool> {
    let Ok(mut child) = Command::new("cvc5")
        .arg("--lang=smt2")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        assert!(
            !is_ci(),
            "smt_verify: `cvc5` not found on PATH in CI. It should have been \
             installed by the CI workflow (see .github/workflows/test-rust-workspace.yml) \
             — this must not silently skip in CI."
        );
        eprintln!("skipping smt_verify test: `cvc5` not found on PATH");
        return None;
    };

    child
        .stdin
        .take()
        .expect("child was spawned with a piped stdin")
        .write_all(script.as_bytes())
        .expect("failed to write SMT-LIB2 script to cvc5's stdin");

    let output = child.wait_with_output().expect("failed to read cvc5's output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let verdict = stdout.lines().next().unwrap_or("").trim();

    match verdict {
        "unsat" => Some(true),
        "sat" => Some(false),
        other => panic!("unexpected cvc5 output: {other:?}\nfull output:\n{stdout}"),
    }
}
