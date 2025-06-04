fn case1() -> [Field] {
    if true {}
    &[1]
}

fn case2() -> [u8] {
    let mut var: u8 = 1;
    {
        var += 1;
    }
    &[var]
}

fn main() {
    let _ = case1();
    let _ = case2();
}
