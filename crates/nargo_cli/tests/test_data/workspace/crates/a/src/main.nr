fn main(x : Field, y : pub Field) {
    assert(x != y);
}

#[test]
fn a() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
