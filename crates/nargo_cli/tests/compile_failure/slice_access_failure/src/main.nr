fn main(x : Field, y : pub Field) {
    let mut slice = [0; 2];
    if x == y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }
    // This constraint should fail as the slice length is 2 and the index is 2
    assert(slice[2] == 0);
}

#[test]
fn test_main() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
