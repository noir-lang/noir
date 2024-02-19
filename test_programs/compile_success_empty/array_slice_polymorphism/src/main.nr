
fn array_zeros<N?>(_x: [u64; N]) -> [u64; N] {
    [0; N]
}

fn main(x: Field, y: pub Field) {
    let xs: [u64] = [11111; 1337]; 
    assert(array_zeros(xs) == xs);
    assert(x != y);
}

#[test]
fn test_main() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
