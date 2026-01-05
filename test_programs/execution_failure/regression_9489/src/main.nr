global G_A: [str<0>] = #[];
fn main() -> pub bool {
    let _ = G_A.pop_front().0;
    true
}
