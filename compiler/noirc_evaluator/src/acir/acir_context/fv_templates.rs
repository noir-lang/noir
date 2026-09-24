//! Pins the constraints emitted by the integer gadgets against the templates
//! whose soundness is proved in Lean (`fv/acir_lean`). The golden file is
//! produced by `EmitTemplates.lean` (see `fv/acir_lean/README.md`); a change to
//! either side fails here until the proof and the Rust agree again.
//!
//! REVIEWED: this file is the Rust half of the pin, so it is part of the trusted
//! base. `canonical` must print every constraint faithfully, in the same form as
//! `fv/acir_lean/AcirLean/Spec/Pin.lean`, and the gadget calls below must cover
//! every width in `pinnedWidths` there. `signed_lt` prints the SSA with `Ssa`'s own
//! `Display`, which `Spec/Ssa.lean` mirrors for the instructions it uses, and
//! `acir_of` and `shipped_of` must print the full output of ACIR generation and
//! of the optimized circuit, including the input and return witnesses.

use acvm::{
    AcirField, FieldElement,
    acir::circuit::Opcode,
    acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
};
use num_bigint::BigUint;

use super::{AcirContext, BrilligStdLib};
use crate::brillig::BrilligOptions;
use crate::ssa::ssa_gen::Ssa;

/// One constraint in the canonical text form shared with the Lean emitter:
/// `zero c*[i,j] + c*[i] + c*[]` (terms sorted by witness list, merged, zero
/// terms dropped, sign chosen so the first coefficient is at most (p-1)/2), or
/// `range i k`. Brillig calls are skipped: they add no constraints.
fn canonical(opcodes: &[Opcode<FieldElement>]) -> Vec<String> {
    let modulus = FieldElement::modulus();
    let half = (&modulus - 1u32) / 2u32;
    let mut out = Vec::new();
    for opcode in opcodes {
        match opcode {
            Opcode::AssertZero(expr) => {
                let mut terms: Vec<(Vec<u32>, FieldElement)> = Vec::new();
                let mut push = |mut ws: Vec<u32>, c: FieldElement| {
                    ws.sort();
                    if let Some(t) = terms.iter_mut().find(|t| t.0 == ws) {
                        t.1 += c;
                    } else {
                        terms.push((ws, c));
                    }
                };
                for (c, a, b) in &expr.mul_terms {
                    push(vec![a.0, b.0], *c);
                }
                for (c, w) in &expr.linear_combinations {
                    push(vec![w.0], *c);
                }
                push(vec![], expr.q_c);
                terms.retain(|t| !t.1.is_zero());
                terms.sort_by(|a, b| a.0.cmp(&b.0));
                if let Some(first) = terms.first()
                    && BigUint::from_bytes_be(&first.1.to_be_bytes()) > half
                {
                    for t in &mut terms {
                        t.1 = -t.1;
                    }
                }
                let body: Vec<String> = terms
                    .iter()
                    .map(|(ws, c)| {
                        let ws: Vec<String> = ws.iter().map(u32::to_string).collect();
                        format!("{}*[{}]", BigUint::from_bytes_be(&c.to_be_bytes()), ws.join(","))
                    })
                    .collect();
                out.push(format!("zero {}", body.join(" + ")));
            }
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(w),
                num_bits,
            }) => out.push(format!("range {} {}", w.0, num_bits)),
            Opcode::BrilligCall { .. } => {}
            other => out.push(format!("other {other:?}")),
        }
    }
    out
}

fn div_var(bit_size: u32) -> Vec<String> {
    let mut context = AcirContext::<FieldElement>::new(BrilligStdLib::default());
    let lhs = context.add_variable();
    let rhs = context.add_variable();
    let one = context.add_constant(FieldElement::one());
    context.euclidean_division_var(lhs, rhs, bit_size, one).unwrap();
    canonical(context.acir_ir.opcodes())
}

fn truncate_field(bits: u32) -> Vec<String> {
    let mut context = AcirContext::<FieldElement>::new(BrilligStdLib::default());
    let lhs = context.add_variable();
    context.truncate_var(lhs, bits, FieldElement::max_num_bits()).unwrap();
    canonical(context.acir_ir.opcodes())
}

fn div_var_predicated(bit_size: u32) -> Vec<String> {
    let mut context = AcirContext::<FieldElement>::new(BrilligStdLib::default());
    let lhs = context.add_variable();
    let rhs = context.add_variable();
    let predicate = context.add_variable();
    context.euclidean_division_var(lhs, rhs, bit_size, predicate).unwrap();
    canonical(context.acir_ir.opcodes())
}

fn more_than_eq(bit_size: u32) -> Vec<String> {
    let mut context = AcirContext::<FieldElement>::new(BrilligStdLib::default());
    let lhs = context.add_variable();
    let rhs = context.add_variable();
    context.more_than_eq_var(lhs, rhs, bit_size).unwrap();
    canonical(context.acir_ir.opcodes())
}

/// The SSA `expand_signed_math` produces for a signed `lt`, as printed.
fn signed_lt(bit_size: u32) -> Vec<String> {
    let src = format!(
        "acir(inline) fn main f0 {{\n  b0(v0: i{bit_size}, v1: i{bit_size}):\n    v2 = lt v0, v1\n    return v2\n}}\n"
    );
    let ssa = Ssa::from_str(&src).unwrap().expand_signed_math();
    ssa.to_string().trim().lines().map(str::to_string).collect()
}

/// The ACIR that ACIR generation emits for an SSA function, before optimization:
/// the canonical constraints, then the input and return witnesses.
fn acir_of(src: &str) -> Vec<String> {
    acir_of_ssa(Ssa::from_str(src).unwrap())
}

fn acir_of_ssa(ssa: Ssa) -> Vec<String> {
    let brillig = ssa.to_brillig(&BrilligOptions::default());
    let (acirs, _, _) = ssa.into_acir(&brillig, &BrilligOptions::default()).unwrap();
    let acir = &acirs[0];
    let mut lines = canonical(acir.opcodes());
    let witnesses = |ws: &[acvm::acir::native_types::Witness]| {
        ws.iter().map(|w| w.0.to_string()).collect::<Vec<_>>().join(",")
    };
    lines.push(format!("inputs [{}]", witnesses(&acir.input_witnesses)));
    lines.push(format!("returns [{}]", witnesses(&acir.return_witnesses)));
    lines
}

/// The optimized circuit for an SSA function: what `nargo compile` ships.
fn shipped_of(src: &str) -> Vec<String> {
    shipped_of_ssa(Ssa::from_str(src).unwrap())
}

fn shipped_of_ssa(ssa: Ssa) -> Vec<String> {
    let (program, _) = crate::acir::tests::try_ssa_value_to_acir(ssa).unwrap();
    let circuit = &program.functions[0];
    let mut lines = canonical(&circuit.opcodes);
    let witnesses = |ws: Vec<u32>| ws.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
    lines.push(format!(
        "inputs [{}]",
        witnesses(circuit.private_parameters.iter().map(|w| w.0).collect())
    ));
    lines.push(format!(
        "returns [{}]",
        witnesses(circuit.return_values.0.iter().map(|w| w.0).collect())
    ));
    lines
}

/// The optimized circuit for a signed `div` or `mod` on `i<n>` after
/// `expand_signed_math`.
fn shipped_signed(op: &str, bit_size: u32) -> Vec<String> {
    let src = format!(
        "acir(inline) fn main f0 {{\n  b0(v0: i{bit_size}, v1: i{bit_size}):\n    v2 = {op} v0, v1\n    return v2\n}}\n"
    );
    shipped_of_ssa(Ssa::from_str(&src).unwrap().expand_signed_math())
}

/// `v<result> = <op> v<a>, v<b>`.
type Instruction = (&'static str, usize, usize);

/// The straight-line programs the Lean checker is run on: `u<n>` parameters `v0`,
/// `v1`, then `div` and `lt` instructions whose operands are earlier `u<n>` values,
/// returning the last result. Operands are always two different values.
fn corpus_programs() -> Vec<(u32, Vec<Instruction>)> {
    let mut programs = Vec::new();
    for width in [8, 64, 128] {
        for op in ["div", "lt"] {
            programs.push((width, vec![(op, 0, 1)]));
            programs.push((width, vec![(op, 1, 0)]));
        }
        for (a, b) in [(0, 1), (1, 0)] {
            for (c, d) in [(2, 0), (0, 2), (2, 1), (1, 2)] {
                for op in ["div", "lt"] {
                    programs.push((width, vec![("div", a, b), (op, c, d)]));
                }
            }
        }
        programs.push((width, vec![("div", 0, 1), ("div", 2, 1), ("lt", 3, 0)]));
        programs.push((width, vec![("div", 1, 0), ("div", 0, 2), ("div", 3, 1)]));
    }
    programs
}

fn corpus_source(width: u32, body: &[Instruction]) -> String {
    let mut src = format!("acir(inline) fn main f0 {{\n  b0(v0: u{width}, v1: u{width}):\n");
    for (i, (op, a, b)) in body.iter().enumerate() {
        src.push_str(&format!("    v{} = {op} v{a}, v{b}\n", i + 2));
    }
    src.push_str(&format!("    return v{}\n}}\n", body.len() + 1));
    src
}

/// The shipped circuit for a corpus program, and the witness ACVM solves for it on
/// the first sample input it accepts.
fn corpus_entry(width: u32, body: &[Instruction]) -> Vec<String> {
    use acvm::{
        acir::native_types::{Witness, WitnessMap},
        blackbox_solver::StubbedBlackBoxSolver,
        pwg::{ACVM, ACVMStatus},
    };
    let src = corpus_source(width, body);
    let (program, _) = crate::acir::tests::try_ssa_to_acir(&src).unwrap();
    let circuit = &program.functions[0];
    let mut lines: Vec<String> = src.trim().lines().map(str::to_string).collect();
    lines.extend(shipped_of(&src));
    let big = FieldElement::from(2u128).pow(&FieldElement::from(u128::from(width - 1)))
        + FieldElement::from(5u128);
    let candidates = [
        (big, FieldElement::from(7u128)),
        (FieldElement::from(7u128), big),
        (FieldElement::from(13u128), FieldElement::from(5u128)),
    ];
    let solver = StubbedBlackBoxSolver;
    let mut acvm = None;
    for (a, b) in candidates {
        let mut inputs = WitnessMap::new();
        inputs.insert(Witness(0), a);
        inputs.insert(Witness(1), b);
        let mut vm =
            ACVM::new(&solver, &circuit.opcodes, inputs, &program.unconstrained_functions, &[]);
        if vm.solve() == ACVMStatus::Solved {
            acvm = Some(vm);
            break;
        }
    }
    let acvm = acvm.expect("some candidate input satisfies the program");
    for (w, v) in acvm.witness_map().clone() {
        lines.push(format!("witness {} {}", w.0, BigUint::from_bytes_be(&v.to_be_bytes())));
    }
    lines
}

fn emitted() -> String {
    let mut sections = Vec::new();
    for n in [8, 16, 32, 64, 128] {
        sections.push(format!("# div_var {n}\n{}", div_var(n).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        sections.push(format!("# div_var_predicated {n}\n{}", div_var_predicated(n).join("\n")));
    }
    for k in [8, 16, 32, 64, 128] {
        sections.push(format!("# truncate_field {k}\n{}", truncate_field(k).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        sections.push(format!("# more_than_eq {n}\n{}", more_than_eq(n).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        sections.push(format!("# signed_lt {n}\n{}", signed_lt(n).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let src = format!(
            "acir(inline) fn main f0 {{\n  b0(v0: u{n}, v1: u{n}):\n    v2 = div v0, v1\n    return v2\n}}\n"
        );
        sections.push(format!("# acir_div {n}\n{}", acir_of(&src).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let src = format!(
            "acir(inline) fn main f0 {{\n  b0(v0: u{n}, v1: u{n}):\n    v2 = lt v0, v1\n    return v2\n}}\n"
        );
        sections.push(format!("# acir_lt {n}\n{}", acir_of(&src).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let src = format!(
            "acir(inline) fn main f0 {{\n  b0(v0: Field):\n    v1 = truncate v0 to {n} bits, max_bit_size: 254\n    v2 = cast v1 as u{n}\n    return v2\n}}\n"
        );
        sections.push(format!("# acir_truncate {n}\n{}", acir_of(&src).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let ssa = signed_lt(n).join("\n") + "\n";
        sections.push(format!("# acir_signed_lt {n}\n{}", acir_of(&ssa).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let src = format!(
            "acir(inline) fn main f0 {{\n  b0(v0: u{n}, v1: u{n}):\n    v2 = div v0, v1\n    return v2\n}}\n"
        );
        sections.push(format!("# shipped_div {n}\n{}", shipped_of(&src).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let src = format!(
            "acir(inline) fn main f0 {{\n  b0(v0: u{n}, v1: u{n}):\n    v2 = lt v0, v1\n    return v2\n}}\n"
        );
        sections.push(format!("# shipped_lt {n}\n{}", shipped_of(&src).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let src = format!(
            "acir(inline) fn main f0 {{\n  b0(v0: Field):\n    v1 = truncate v0 to {n} bits, max_bit_size: 254\n    v2 = cast v1 as u{n}\n    return v2\n}}\n"
        );
        sections.push(format!("# shipped_truncate {n}\n{}", shipped_of(&src).join("\n")));
    }
    for n in [8, 16, 32, 64, 128] {
        let ssa = signed_lt(n).join("\n") + "\n";
        sections.push(format!("# shipped_signed_lt {n}\n{}", shipped_of(&ssa).join("\n")));
    }
    for n in [8, 16, 32, 64] {
        sections.push(format!("# shipped_signed_div {n}\n{}", shipped_signed("div", n).join("\n")));
    }
    for n in [8, 16, 32, 64] {
        sections.push(format!("# shipped_signed_mod {n}\n{}", shipped_signed("mod", n).join("\n")));
    }
    for (i, (width, body)) in corpus_programs().iter().enumerate() {
        sections.push(format!("# corpus {i}\n{}", corpus_entry(*width, body).join("\n")));
    }
    sections.join("\n") + "\n"
}

#[test]
fn integer_gadgets_match_lean_templates() {
    let golden =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fv/acir_lean/templates.golden"));
    let emitted = emitted();
    if emitted != golden {
        panic!(
            "integer gadget constraints differ from the Lean-proved templates.\n\
             templates.golden is generated from fv/acir_lean and checked by \
             fv/acir_lean/scripts/check.sh: editing it by hand fails that check. \
             Update AcirLean/Templates/Gadgets.lean and the proofs to match the new constraints, \
             then regenerate the golden file from Lean.\n\
             --- emitted ---\n{emitted}"
        );
    }
}

/// Prints, for each `nargo` artifact listed in the file named by `FV_ARTIFACTS`,
/// the canonical constraints of its main circuit and its input and return
/// witnesses. Used to regenerate `fv/acir_lean` test-program data.
#[test]
#[ignore = "run by fv/acir_lean/scripts/regen_programs.sh"]
fn dump_artifacts() {
    let list = std::env::var("FV_ARTIFACTS").unwrap();
    for path in std::fs::read_to_string(list).unwrap().lines() {
        let json = std::fs::read_to_string(path).unwrap();
        let artifact: noirc_artifacts::program::ProgramArtifact =
            serde_json::from_str(&json).unwrap();
        let circuit = &artifact.bytecode.functions[0];
        let mut inputs: Vec<u32> = circuit.private_parameters.iter().map(|w| w.0).collect();
        inputs.extend(circuit.public_parameters.0.iter().map(|w| w.0));
        inputs.sort_unstable();
        let returns: Vec<u32> = circuit.return_values.0.iter().map(|w| w.0).collect();
        let join = |ws: &[u32]| ws.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        println!("# artifact {path}");
        for line in canonical(&circuit.opcodes) {
            println!("{line}");
        }
        println!("inputs [{}]", join(&inputs));
        println!("returns [{}]", join(&returns));
    }
}
