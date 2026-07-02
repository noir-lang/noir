fn main(i: u32) -> pub bool {
    let a: [(Field, bool); 2] = [(10, false), (22, true)];
    a[i].1
}
