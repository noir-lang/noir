fn main(x: u32) -> pub bool {
    let d: [&mut bool; 1] = [&mut true];
    *d[x]
}
