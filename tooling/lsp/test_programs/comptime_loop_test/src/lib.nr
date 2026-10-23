comptime fn spin() -> u32 {
    let mut i = 0;
    loop {
        i += 1;
        if i == 20000 {
            break;
        }
    }
    i
}

#[test]
fn test_spin() {
    let count = comptime { spin() };
    assert_eq(count, 20000);
}
