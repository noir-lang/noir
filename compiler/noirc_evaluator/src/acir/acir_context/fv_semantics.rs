//! Pins the meaning of SSA instructions assumed by the Lean proofs
//! (`Instruction.step` in `fv/acir_lean/AcirLean/Spec/SsaSemantics.lean`) against Noir's
//! SSA interpreter. `ssa_semantics.golden` is produced by `EmitSemantics.lean`: each
//! one-instruction function followed by the calls made to it and the result Lean
//! computes. Every call is replayed here through the interpreter, and the test fails
//! on any disagreement.
//!
//! REVIEWED: this file is part of the trusted base. `outcome` must print the
//! interpreter's result in the same form as `Case.result` in `EmitSemantics.lean`:
//! `fail` for an error or a panic, `ok` for no return values, otherwise
//! `<type> <value>` per return value.

use std::panic::{AssertUnwindSafe, catch_unwind};

use acvm::{AcirField, FieldElement};
use num_bigint::BigUint;

use crate::ssa::interpreter::value::{NumericValue, Value};
use crate::ssa::ir::types::NumericType;
use crate::ssa::ssa_gen::Ssa;

fn numeric_type(name: &str) -> NumericType {
    match name {
        "Field" => NumericType::NativeField,
        _ => {
            let bit_size = name[1..].parse().unwrap();
            if name.starts_with('u') {
                NumericType::unsigned(bit_size)
            } else {
                assert!(name.starts_with('i'), "unknown type {name}");
                NumericType::signed(bit_size)
            }
        }
    }
}

fn field(value: &str) -> FieldElement {
    FieldElement::from_be_bytes_reduce(&value.parse::<BigUint>().unwrap().to_bytes_be())
}

/// Runs `f`, turning a panic into `Err` without printing it.
fn quietly<T>(f: impl FnOnce() -> T) -> std::thread::Result<T> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(hook);
    result
}

fn outcome(ssa: &Ssa, args: Vec<Value>) -> String {
    // The interpreter asserts that a `u1` is 0 or 1; that panic is a failure here.
    match quietly(|| ssa.interpret(args)) {
        Ok(Ok(values)) if values.is_empty() => "ok".to_string(),
        Ok(Ok(values)) => values
            .iter()
            .map(|value| match value {
                Value::Numeric(n) => format!(
                    "{} {}",
                    n.get_type(),
                    BigUint::from_bytes_be(&n.to_field().to_be_bytes())
                ),
                other => panic!("unexpected return value {other}"),
            })
            .collect::<Vec<_>>()
            .join(", "),
        Ok(Err(_)) | Err(_) => "fail".to_string(),
    }
}

/// `b0(v0: T, ...): <instruction> | ... | return ...` as a one-block ACIR function,
/// or why the SSA parser or validator rejects it.
fn parse(line: &str) -> Result<(Ssa, Vec<NumericType>), String> {
    let (block, rest) = line.split_once("): ").unwrap();
    let params = block.strip_prefix("b0(").unwrap();
    let types = if params.is_empty() {
        Vec::new()
    } else {
        params.split(", ").map(|p| numeric_type(p.split_once(": ").unwrap().1)).collect()
    };
    let body = rest.split(" | ").map(|line| format!("    {line}\n")).collect::<String>();
    let src = format!("acir(inline) fn main f0 {{\n  {block}):\n{body}}}\n");
    match quietly(|| Ssa::from_str(&src)) {
        Ok(Ok(ssa)) => Ok((ssa, types)),
        Ok(Err(error)) => Err(format!("{error:?}")),
        Err(panic) => Err(panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()),
    }
}

#[test]
fn ssa_meaning_in_lean_matches_the_interpreter() {
    let golden = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fv/acir_lean/ssa_semantics.golden"
    ));
    let mut current = None;
    let mut calls = 0;
    let mut mismatches = Vec::new();
    let mut rejected = Vec::new();
    for line in golden.lines() {
        let Some(call) = line.strip_prefix("  ") else {
            current = match parse(line) {
                Ok(parsed) => Some((line, parsed)),
                Err(reason) => {
                    rejected.push(format!("{line}\n  {reason}"));
                    None
                }
            };
            continue;
        };
        let Some((function, (ssa, types))) = current.as_ref() else { continue };
        let (args, expected) = call.split_once(" => ").unwrap();
        let args = args
            .split(", ")
            .zip(types)
            .map(|(arg, typ)| {
                Value::Numeric(NumericValue::int_from_field(field(arg), *typ).unwrap())
            })
            .collect();
        let actual = outcome(ssa, args);
        calls += 1;
        if actual != expected {
            mismatches.push(format!("{function}\n  {call}\n  interpreter: {actual}"));
        }
    }
    assert!(calls > 0, "ssa_semantics.golden has no cases");
    assert!(
        rejected.is_empty(),
        "Noir rejects {} functions in ssa_semantics.golden, so no program contains them. \
         Leave them out of the grid in fv/acir_lean/EmitSemantics.lean:\n{}",
        rejected.len(),
        rejected.join("\n")
    );
    assert!(
        mismatches.is_empty(),
        "The Lean meaning of {} of {calls} SSA calls differs from Noir's SSA interpreter.\n\
         Fix `Instruction.run` or `Instruction.step` in fv/acir_lean/AcirLean/Spec/SsaSemantics.lean (and the proofs), \
         then regenerate with `lake env lean --run EmitSemantics.lean ssa_semantics.golden`.\n\n{}",
        mismatches.len(),
        mismatches.iter().take(30).cloned().collect::<Vec<_>>().join("\n")
    );
}
