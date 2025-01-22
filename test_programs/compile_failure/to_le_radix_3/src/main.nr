fn main() {
    assert(!std::runtime::is_unconstrained(), "expected this test to be run constrained");
    let field = 2;
    let _: [u8; 8] = field.to_le_radix(3);
}
