fn main(x: u32) {
    assert(fibonacci(x) == 34);
}

#[fold]
fn fibonacci(x: u32) -> u32 {
    if x <= 1 {
        x
    } else {
        fibonacci(x - 1) + fibonacci(x - 2)
    }
}
