//! Pins the constraints emitted by the integer gadgets against the templates
//! whose soundness is proved in Lean (`fv/acir_lean`). The golden file is
//! produced by `EmitTemplates.lean` (see `fv/acir_lean/README.md`); a change to
//! either side fails here until the proof and the Rust agree again.
//!
//! REVIEWED: this file is the Rust half of the pin, so it is part of the trusted
//! base. `canonical` must print every constraint faithfully, in the same form as
//! `fv/acir_lean/AcirLean/Spec/Pin.lean`, and the gadget calls below must cover
//! every width in `pinnedWidths` there.

use acvm::{
    AcirField, FieldElement,
    acir::circuit::Opcode,
    acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
};
use num_bigint::BigUint;

use super::{AcirContext, BrilligStdLib};

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

fn emitted() -> String {
    let mut sections = Vec::new();
    for n in [8, 16, 32, 64] {
        sections.push(format!("# div_var {n}\n{}", div_var(n).join("\n")));
    }
    for k in [8, 16, 32, 64] {
        sections.push(format!("# truncate_field {k}\n{}", truncate_field(k).join("\n")));
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
