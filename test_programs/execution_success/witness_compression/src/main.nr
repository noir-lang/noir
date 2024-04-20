// This test should be used to regenerate the serialized witness used in the `acvm_js` integration tests.
// The `acvm_js` test file containing the serialized witness should be also called `witness_compression`.
// After recompiling Noir, you can manually print the witness byte array to be written to file after execution.
fn main(x: field, y: pub field) -> pub field {
    assert(x != y);
    x + y
}
