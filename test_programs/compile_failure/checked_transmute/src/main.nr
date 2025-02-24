use std::mem::checked_transmute;

fn main() {
    let _: [Field; 2] = transmute_fail([1]);
}

pub fn transmute_fail<let N: u32>(x: [Field; N]) -> [Field; N + 1] {
    checked_transmute(x)
}
