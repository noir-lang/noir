fn main() -> pub u8 {
    foo::<3_u32>()
}

fn foo<let N: u8>() -> u8 {
    N
}
