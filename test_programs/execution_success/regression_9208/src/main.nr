global G_A: [Field] = &[];
fn main(a: u32) -> pub Field {
    match a {
        10 => G_A[1_u32 % G_A.len()],
        20 => (a as Field),
        _ => -1,
    }
}
