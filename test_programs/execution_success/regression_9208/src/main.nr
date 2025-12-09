global G_A: [Field] = &[];
fn main(a: u32) -> pub Field {
    if a == 10 {
        G_A[1_u32 % G_A.len()]
    } else if a == 20 {
        (a as Field)
    } else {
        -1
    }
}
