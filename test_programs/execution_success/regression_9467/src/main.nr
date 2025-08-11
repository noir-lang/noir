global G_A: [u32] = &[];
fn main(b: bool) -> pub u32 {
    if b {
        G_A.pop_front().0
    } else {
        1
    }
}
