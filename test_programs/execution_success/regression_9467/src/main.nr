global G_A: [u32] = &[];
fn main(b: bool) -> pub (u32, u32) {
    if b {
        (G_A.pop_front().0, G_A.remove(0).1)
    } else {
        (1, 1)
    }
}
