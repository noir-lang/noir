fn main(input: [u8; 4]) -> pub bool {
    // Secret password (demo only)
    let password: [u8; 4] = [1, 2, 3, 4];
    input == password
}

#[test]
fn correct_password() {
    let attempt: [u8; 4] = [1, 2, 3, 4];
    assert(main(attempt)); // should pass
}

#[test]
fn wrong_password() {
    let attempt: [u8; 4] = [9, 9, 9, 9];
    assert(!main(attempt)); // should fail
}
