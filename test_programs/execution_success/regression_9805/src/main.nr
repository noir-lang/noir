fn main(a: u32, b: bool) -> pub bool {
    let c = [0, 1];
    for _ in 0..2 {
        if b {
            let d = c[a];
            println(d);
        }
    }
    true
}
