fn foo<let N: u32>() -> u32 {
    N
}

fn main() {
    let _ = foo();
}

