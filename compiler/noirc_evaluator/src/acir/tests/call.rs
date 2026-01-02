use acvm::assert_circuit_snapshot;
use noirc_frontend::monomorphization::ast::InlineType;

use crate::acir::tests::ssa_to_acir_program;

/// Check that each `InlineType` which prevents inlining functions generates code in the same manner
#[test]
fn basic_calls_fold() {
    basic_call_with_outputs_assert(InlineType::Fold);
    call_output_as_next_call_input(InlineType::Fold);
    basic_nested_call(InlineType::Fold);
}

#[test]
#[should_panic = "ICE: Got a call to an ACIR function f1 named foo that should have already been inlined"]
fn basic_calls_no_predicates() {
    basic_call_with_outputs_assert(InlineType::NoPredicates);
}

#[test]
#[should_panic = "ICE: Got a call to an ACIR function f1 named foo that should have already been inlined"]
fn call_output_as_next_call_input_no_predicates() {
    call_output_as_next_call_input(InlineType::NoPredicates);
}

#[test]
#[should_panic = "ICE: Got a call to an ACIR function f1 named func_with_nested_foo_call that should have already been inlined"]
fn nested_call_no_predicates() {
    basic_nested_call(InlineType::NoPredicates);
}

#[test]
#[should_panic = "ICE: Got a call to an ACIR function f1 named foo that should have already been inlined"]
fn call_without_inline_attribute() {
    basic_call_with_outputs_assert(InlineType::Inline);
}

fn basic_call_with_outputs_assert(inline_type: InlineType) {
    let src = &format!(
        "
    acir(inline) fn main f0 {{
      b0(v0: Field, v1: Field):
        v3 = call f1(v0, v1) -> Field
        v4 = call f1(v0, v1) -> Field
        constrain v3 == v4
        return
    }}
    acir({inline_type}) fn foo f1 {{
      b0(v0: Field, v1: Field):
        constrain v0 == v1
        return v0
    }}
    "
    );

    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w2]
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w3]
    ASSERT w3 = w2

    func 1
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w1 = w0
    ASSERT w2 = w0
    ");
}

fn call_output_as_next_call_input(inline_type: InlineType) {
    let src = &format!(
        "
    acir(inline) fn main f0 {{
      b0(v0: Field, v1: Field):
        v3 = call f1(v0, v1) -> Field
        v4 = call f1(v3, v1) -> Field
        constrain v3 == v4
        return
    }}
    acir({inline_type}) fn foo f1 {{
      b0(v0: Field, v1: Field):
        constrain v0 == v1
        return v0
    }}
    "
    );

    let program = ssa_to_acir_program(src);
    // The expected result should look very similar to the `basic_call_with_outputs_assert test except that
    // the input witnesses of the `Call` opcodes will be different. The differences can discerned from the output below.
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w2]
    CALL func: 1, predicate: 1, inputs: [w2, w1], outputs: [w3]
    ASSERT w3 = w2

    func 1
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w1 = w0
    ASSERT w2 = w0
    ");
}

fn basic_nested_call(inline_type: InlineType) {
    let src = &format!(
        "
    acir(inline) fn main f0 {{
      b0(v0: Field, v1: Field):
        v3 = call f1(v0, v1) -> Field
        v4 = call f1(v0, v1) -> Field
        constrain v3 == v4
        return
    }}
    acir({inline_type}) fn func_with_nested_foo_call f1 {{
      b0(v0: Field, v1: Field):
        v3 = add v0, Field 2
        v5 = call f2(v3, v1) -> Field
        return v5
    }}
    acir({inline_type}) fn foo f2 {{
      b0(v0: Field, v1: Field):
        constrain v0 == v1
        return v0
    }}
    "
    );

    let program = ssa_to_acir_program(src);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w2]
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w3]
    ASSERT w3 = w2

    func 1
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w3 = w0 + 2
    CALL func: 2, predicate: 1, inputs: [w3, w1], outputs: [w4]
    ASSERT w2 = w4

    func 2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    ASSERT w1 = w0
    ASSERT w2 = w0
    ");
}

#[test]
fn call_with_predicate() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: u1):
        enable_side_effects v1
        v2 = call f1(v0) -> Field
        return v2
    }

    acir(fold) fn one f1 {
      b0(v0: Field):
        v1 = add v0, Field 1
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w1, bits: 1
    CALL func: 1, predicate: w1, inputs: [w0], outputs: [w3]
    ASSERT w2 = w3

    func 1
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    ASSERT w1 = w0 + 1
    ");
}
