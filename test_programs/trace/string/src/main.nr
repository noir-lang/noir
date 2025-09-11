fn main(x: Field, y: pub Field) {
    assert(x != y);
    let qstr: str<13> = "Hello, world!";
}

#[test]
fn test_main() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
