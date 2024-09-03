fn foo<let N: u32>() -> u64 {
    N as u64
}

global J: u64 = 10;

fn bar<let N: u64>() -> u64 {
    foo::<J>()
}

global M: u64 = 3;

fn main() {
    let _ = bar::<M>();
}
