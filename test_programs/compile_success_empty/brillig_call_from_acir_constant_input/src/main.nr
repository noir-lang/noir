// Regression from issue #7372: (https://github.com/noir-lang/noir/issues/7372)
fn main() {
    bar(0)
}

pub unconstrained fn foo(input: Field) -> Field {
    (input == 0) as Field
}

pub fn bar(input: Field) {
    // Safety: test
    let output = unsafe { foo(input) };

    // This test requires an assert message.
    // Non-equality can potentially be a no-op during ACIR gen.
    // We want to make sure that we are appropriately indexing our
    // assertion payload opcode location in the case our first opcode
    // is constraining non-equality and resulted in a no-op.
    assert(output != 0, "");
}
