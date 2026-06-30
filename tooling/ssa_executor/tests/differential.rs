//! Phase 0 differential harness for the SSA interpreter's integer model.
//!
//! Runs the same SSA through three engines and compares the results:
//!   - the SSA interpreter (`Ssa::interpret`),
//!   - ACIR, compiled and solved via ACVM,
//!   - Brillig, compiled and solved via ACVM.
//!
//! The interpreter is meant to be a faithful reference for whichever backend a function targets.
//! This harness surfaces where it isn't — in particular the overflow/underflow corners of the
//! `Fitted`/`Unfit` model that produced noir-lang/noir-claude#1430 and #1441.
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
        Value::Numeric(numeric) => numeric.convert_to_field(),
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

            // ACIR genuinely ICEs on `truncate`-of-unchecked-signed-`sub` (a pre-existing ACIR-gen
            // bug, unrelated to the interpreter): there is no backend answer to match there, so a
            // divergence is only counted when ACIR did *not* crash.
            let acir_ok = interp_acir == acir || acir == Outcome::Panicked;
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

/// Probes feeding an out-of-range (`Unfit`) value into the consumers that historically `unreachable!`
/// on `Unfit` operands (`lt`, `and`, `cast`, `truncate`), to see whether the interpreter panics
/// where the backends compute. The producer is `unchecked_mul i32 i32::MAX, 2` (overflows).
#[test]
fn interpreter_vs_backends_on_unfit_consumers() {
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
