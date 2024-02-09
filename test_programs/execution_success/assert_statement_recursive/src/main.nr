// Tests a very simple program.
// 
// The features being tested is assertion
// This is the same as the `assert_statement` test except we specify
// that the backend should use a prover which will construct proofs
// friendly to recursive verification in another SNARK.
#[recursive]
fn main(x: Field, y: pub Field) {
    assert(x == y, "x and y are not equal");
    assert_eq(x, y, "x and y are not equal");
}