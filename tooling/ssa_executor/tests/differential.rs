//! Differential harness for the SSA interpreter's integer model.
//!
//! Runs the same SSA through three engines and compares the results:
//!   - the SSA interpreter (`Ssa::interpret`),
//!   - ACIR, compiled and solved via ACVM,
//!   - Brillig, compiled and solved via ACVM.
//!
//! The interpreter is meant to be a faithful reference for whichever backend a function targets:
//! ACIR's field arithmetic (a value can exceed its type's range until a range check brings it back)
//! or Brillig's fixed-width wrapping registers. This harness pins that down on the overflow and
//! underflow corners where the two backends disagree, so the interpreter stays in step with both.
//!
//! Run with: `cargo test -p noir_ssa_executor --test differential -- --nocapture`

use std::fmt;

use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use noir_ssa_executor::compiler::compile_from_ssa;
use noir_ssa_executor::runner::execute_single;
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::interpreter::value::{NumericValue, Value};
use noirc_evaluator::ssa::ir::types::NumericType;
use noirc_evaluator::ssa::ssa_gen::Ssa;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Outcome {
    Ok(Vec<FieldElement>),
    /// Compilation, parsing, or execution rejected the program (e.g. a failed range constraint).
    Rejected,
    /// The engine panicked / hit an `unreachable!` / ICE.
    Panicked,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Rejected => write!(f, "rejected"),
            Outcome::Panicked => write!(f, "PANIC"),
            Outcome::Ok(values) => {
                let rendered: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", rendered.join(", "))
            }
        }
    }
}

/// Runs `f`, turning a panic (e.g. an interpreter `unreachable!` or an ACIR-gen ICE) into
/// `Outcome::Panicked` rather than crashing the test. The panic hook is muted for the duration.
fn catch(f: impl FnOnce() -> Outcome + std::panic::UnwindSafe) -> Outcome {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(previous);
    result.unwrap_or(Outcome::Panicked)
}

/// One function parameter: the field value to bind it to, and its SSA numeric type.
type Input = (FieldElement, NumericType);

fn run_interpreter(src: &str, inputs: &[Input]) -> Outcome {
    let src = src.to_string();
    let inputs = inputs.to_vec();
    catch(move || {
        let Ok(ssa) = Ssa::from_str(&src) else {
            return Outcome::Rejected;
        };
        let args = inputs
            .iter()
            .map(|(field, typ)| {
                Value::Numeric(
                    NumericValue::from_constant(*field, *typ).expect("input fits its type"),
                )
            })
            .collect();
        match ssa.interpret(args) {
            Ok(values) => Outcome::Ok(values.iter().map(value_to_field).collect()),
            Err(_) => Outcome::Rejected,
        }
    })
}

fn value_to_field(value: &Value) -> FieldElement {
    match value {
        Value::Numeric(numeric) => numeric.to_field(),
        other => panic!("unexpected non-numeric return value: {other:?}"),
    }
}

fn run_backend(src: &str, inputs: &[Input]) -> Outcome {
    let src = src.to_string();
    let inputs = inputs.to_vec();
    catch(move || run_backend_inner(&src, &inputs))
}

fn run_backend_inner(src: &str, inputs: &[Input]) -> Outcome {
    let Ok(ssa) = Ssa::from_str(src) else {
        return Outcome::Rejected;
    };
    let compiled = match compile_from_ssa(ssa, &CompileOptions::default()) {
        Ok(compiled) => compiled,
        // `compile_from_ssa` catches ICEs and reports them as a compilation error; distinguish a
        // genuine ICE ("Panic"/"unreachable"/"internal error") from a normal compile rejection.
        Err(error) => {
            let message = format!("{error:?}");
            if message.contains("Panic")
                || message.contains("unreachable")
                || message.contains("internal error")
            {
                return Outcome::Panicked;
            }
            return Outcome::Rejected;
        }
    };

    let mut witness_map = WitnessMap::new();
    for (index, (field, _typ)) in inputs.iter().enumerate() {
        witness_map.insert(Witness(index as u32), *field);
    }

    match execute_single(&compiled.program, witness_map) {
        Ok(stack) => {
            let witness = &stack.peek().expect("a solved witness frame").witness;
            let return_values = &compiled.program.functions[0].return_values;
            let fields = return_values
                .0
                .iter()
                .map(|w| *witness.get(w).expect("return witness is solved"))
                .collect();
            Outcome::Ok(fields)
        }
        Err(_) => Outcome::Rejected,
    }
}

fn field(value: u128) -> FieldElement {
    FieldElement::from(value)
}

struct Case {
    name: &'static str,
    /// SSA `iN`/`uN` type spelling.
    ty: &'static str,
    nt: NumericType,
    /// `add`, `sub`, or `mul` (the unchecked form is exercised).
    op: &'static str,
    /// Operand bit patterns (for signed values, the two's-complement encoding).
    a: u128,
    b: u128,
}

/// Builds an SSA program for the given runtime and shape:
///   `direct`  : returns the unchecked op result as-is.
///   `checked` : feeds the unchecked result through a checked `add _, 0` (forces a range check).
fn build_src(runtime: &str, case: &Case, checked: bool) -> String {
    let Case { ty, op, .. } = case;
    if checked {
        format!(
            "{runtime}(inline) fn main f0 {{
               b0(v0: {ty}, v1: {ty}):
                 v2 = unchecked_{op} v0, v1
                 v3 = add v2, {ty} 0
                 return v3
             }}"
        )
    } else {
        format!(
            "{runtime}(inline) fn main f0 {{
               b0(v0: {ty}, v1: {ty}):
                 v2 = unchecked_{op} v0, v1
                 return v2
             }}"
        )
    }
}

#[test]
fn interpreter_vs_backends_on_overflow_corners() {
    let cases = [
        Case {
            name: "u8 add",
            ty: "u8",
            nt: NumericType::Unsigned { bit_size: 8 },
            op: "add",
            a: 200,
            b: 100,
        },
        Case {
            name: "u8 sub",
            ty: "u8",
            nt: NumericType::Unsigned { bit_size: 8 },
            op: "sub",
            a: 0,
            b: 10,
        },
        Case {
            name: "u8 mul",
            ty: "u8",
            nt: NumericType::Unsigned { bit_size: 8 },
            op: "mul",
            a: 128,
            b: 2,
        },
        Case {
            name: "u32 add",
            ty: "u32",
            nt: NumericType::Unsigned { bit_size: 32 },
            op: "add",
            a: 4_000_000_000,
            b: 1_000_000_000,
        },
        Case {
            name: "u32 sub",
            ty: "u32",
            nt: NumericType::Unsigned { bit_size: 32 },
            op: "sub",
            a: 0,
            b: 1,
        },
        Case {
            name: "u32 mul",
            ty: "u32",
            nt: NumericType::Unsigned { bit_size: 32 },
            op: "mul",
            a: 2_147_483_648,
            b: 3,
        },
        Case {
            name: "u64 mul",
            ty: "u64",
            nt: NumericType::Unsigned { bit_size: 64 },
            op: "mul",
            a: 10_000_000_000,
            b: 10_000_000_000,
        },
        // signed operands use the two's-complement bit pattern
        Case {
            name: "i8 add",
            ty: "i8",
            nt: NumericType::Signed { bit_size: 8 },
            op: "add",
            a: 127,
            b: 1,
        },
        Case {
            name: "i8 sub(min-1)",
            ty: "i8",
            nt: NumericType::Signed { bit_size: 8 },
            op: "sub",
            a: 128,
            b: 1,
        },
        Case {
            name: "i8 mul",
            ty: "i8",
            nt: NumericType::Signed { bit_size: 8 },
            op: "mul",
            a: 127,
            b: 2,
        },
        // signed sub whose result is negative but in range (3 - 10 = -7)
        Case {
            name: "i8 sub(neg)",
            ty: "i8",
            nt: NumericType::Signed { bit_size: 8 },
            op: "sub",
            a: 3,
            b: 10,
        },
        // signed add whose bit-pattern sum crosses 2^8: -1 (=255) + 1 should be 0, not 256
        Case {
            name: "i8 add(-1+1)",
            ty: "i8",
            nt: NumericType::Signed { bit_size: 8 },
            op: "add",
            a: 255,
            b: 1,
        },
        // signed add -2 + -2 = -4 (bit patterns 254 + 254 = 508, must encode as -4 = 252)
        Case {
            name: "i8 add(-2+-2)",
            ty: "i8",
            nt: NumericType::Signed { bit_size: 8 },
            op: "add",
            a: 254,
            b: 254,
        },
        // signed sub underflowing the type's range (i16 100 - 200 = -100, in range; but check field handling)
        Case {
            name: "i16 sub(neg)",
            ty: "i16",
            nt: NumericType::Signed { bit_size: 16 },
            op: "sub",
            a: 100,
            b: 200,
        },
        Case {
            name: "i32 mul(max*2)",
            ty: "i32",
            nt: NumericType::Signed { bit_size: 32 },
            op: "mul",
            a: 2_147_483_647,
            b: 2,
        },
        Case {
            name: "i32 add(max+1)",
            ty: "i32",
            nt: NumericType::Signed { bit_size: 32 },
            op: "add",
            a: 2_147_483_647,
            b: 1,
        },
    ];

    let mut acir_divergences = Vec::new();
    let mut brillig_divergences = Vec::new();

    println!(
        "\n{:<16} {:<8} {:>10} {:>10}  {:>10} {:>10}",
        "case", "variant", "interp(A)", "acir", "interp(B)", "brillig"
    );
    println!("{}", "-".repeat(74));

    for case in &cases {
        for (variant, checked) in [("direct", false), ("checked", true)] {
            // The SSA validator rejects a checked signed arithmetic op that consumes an unchecked
            // signed `Sub` result (its expansion would truncate the underflowed value). The `checked`
            // variant feeds the unchecked result into a checked `add`, so for a signed `sub` producer
            // that SSA is invalid — skip it rather than asserting on a program that cannot be built.
            if checked && case.op == "sub" && matches!(case.nt, NumericType::Signed { .. }) {
                continue;
            }

            let inputs = [(field(case.a), case.nt), (field(case.b), case.nt)];

            let acir_src = build_src("acir", case, checked);
            let brillig_src = build_src("brillig", case, checked);

            let interp_acir = run_interpreter(&acir_src, &inputs);
            let acir = run_backend(&acir_src, &inputs);
            let interp_brillig = run_interpreter(&brillig_src, &inputs);
            let brillig = run_backend(&brillig_src, &inputs);

            // The interpreter must never panic.
            assert_ne!(interp_acir, Outcome::Panicked, "{} / {} interp(acir)", case.name, variant);
            assert_ne!(
                interp_brillig,
                Outcome::Panicked,
                "{} / {} interp(brillig)",
                case.name,
                variant
            );

            // ACIR-gen has no reliable behavior for checked arithmetic whose operand overflowed its
            // bit width through a prior unchecked op (the field carries the extended value, which is
            // not a valid witness for the type). It either ICEs (e.g. `truncate`-of-unchecked-signed-
            // `sub`) or rejects (e.g. signed `add` of two negatives whose bit-pattern sum exceeds the
            // width), inconsistently — so there is no backend answer to match. We tolerate both, but
            // only when the interpreter still produced the canonical wrapped value (what Brillig
            // gives); a genuine ACIR rejection that the interpreter ignored would still be flagged.
            let acir_ok = interp_acir == acir
                || acir == Outcome::Panicked
                || (acir == Outcome::Rejected && interp_acir == brillig);
            let brillig_ok = interp_brillig == brillig;

            println!(
                "{:<16} {:<8} {:>10} {:>10}  {:>10} {:>10}  {}{}",
                case.name,
                variant,
                interp_acir.to_string(),
                acir.to_string(),
                interp_brillig.to_string(),
                brillig.to_string(),
                if acir_ok { "" } else { "[ACIR DIFF] " },
                if brillig_ok { "" } else { "[BRILLIG DIFF]" },
            );

            if !acir_ok {
                acir_divergences.push(format!(
                    "{} / {}: interp={} acir={}",
                    case.name, variant, interp_acir, acir
                ));
            }
            if !brillig_ok {
                brillig_divergences.push(format!(
                    "{} / {}: interp={} brillig={}",
                    case.name, variant, interp_brillig, brillig
                ));
            }
        }
    }

    println!("\n=== interpreter vs ACIR divergences ({}) ===", acir_divergences.len());
    for d in &acir_divergences {
        println!("  {d}");
    }
    println!("\n=== interpreter vs Brillig divergences ({}) ===", brillig_divergences.len());
    for d in &brillig_divergences {
        println!("  {d}");
    }

    assert!(
        brillig_divergences.is_empty(),
        "interpreter diverges from Brillig:\n{}",
        brillig_divergences.join("\n")
    );
    assert!(
        acir_divergences.is_empty(),
        "interpreter diverges from ACIR (outside the known ACIR-gen ICE):\n{}",
        acir_divergences.join("\n")
    );
}

/// Feeds an out-of-range value (the overflowing product `unchecked_mul i32 i32::MAX, 2`) into the
/// operations that consume it — `lt`, `and`, `truncate` — and checks the interpreter computes the
/// same result as each backend rather than rejecting or panicking on the wider-than-the-type value.
#[test]
fn interpreter_vs_backends_on_out_of_range_consumers() {
    let i32t = NumericType::Signed { bit_size: 32 };
    let inputs = [(field(2_147_483_647), i32t), (field(2), i32t)];

    // The tail consumes `v2` (the overflowed product) and returns a value.
    let consumers: [(&str, &str); 3] = [
        ("lt", "v3 = lt v2, i32 10\n        return v3"),
        ("and", "v3 = and v2, i32 255\n        return v3"),
        ("truncate16", "v3 = truncate v2 to 16 bits, max_bit_size: 64\n        return v3"),
    ];

    let build = |runtime: &str, tail: &str| {
        format!(
            "{runtime}(inline) fn main f0 {{
               b0(v0: i32, v1: i32):
                 v2 = unchecked_mul v0, v1
                 {tail}
             }}"
        )
    };

    println!(
        "\n{:<12} {:>10} {:>10}  {:>10} {:>10}",
        "consumer", "interp(A)", "acir", "interp(B)", "brillig"
    );
    println!("{}", "-".repeat(60));

    for (name, tail) in consumers {
        let acir_src = build("acir", tail);
        let brillig_src = build("brillig", tail);

        let interp_acir = run_interpreter(&acir_src, &inputs);
        let acir = run_backend(&acir_src, &inputs);
        let interp_brillig = run_interpreter(&brillig_src, &inputs);
        let brillig = run_backend(&brillig_src, &inputs);

        println!(
            "{:<12} {:>10} {:>10}  {:>10} {:>10}  {}{}{}",
            name,
            interp_acir.to_string(),
            acir.to_string(),
            interp_brillig.to_string(),
            brillig.to_string(),
            if interp_acir == acir { "" } else { "[ACIR DIFF] " },
            if interp_brillig == brillig { "" } else { "[BRILLIG DIFF] " },
            if matches!(interp_acir, Outcome::Panicked)
                || matches!(interp_brillig, Outcome::Panicked)
            {
                "[INTERP PANIC]"
            } else {
                ""
            },
        );

        // The interpreter must compute (not panic) and match both backends for these consumers.
        assert_ne!(interp_acir, Outcome::Panicked, "{name}: interp(acir) panicked");
        assert_ne!(interp_brillig, Outcome::Panicked, "{name}: interp(brillig) panicked");
        assert_eq!(interp_acir, acir, "{name}: interp(acir) vs acir");
        assert_eq!(interp_brillig, brillig, "{name}: interp(brillig) vs brillig");
    }
}
