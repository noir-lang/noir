// Tests that number equality check can be performed.
//
// This produces the Eq instruction, and then constrains the resulting bool to
// equal u1 0, i.e. false. So, in effect, this tests that the Eq instruction
// works.
//
// Expected to fail, because the witnesses are actually equal.
fn main(x: Field, y: Field) {
    assert(x != y);
}
