fn main(a: i1, zero: i1) {
    assert(0 / a == 0);
    assert(a % a == 0);

    // This if conditional is expected to be false
    // We want to test the division appropriately does not error
    // under an inactive predicate
    if a == 0 {
        assert(a / zero == 0);
        assert(a % zero == 0);
    }
}
