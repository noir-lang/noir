fn main(a: u32, b: bool) -> pub Field {
    let mut s = 0;
    let c = [[1, 2]];
    for _ in 0..2 {
        if b {
            let d = c[a];
            let e = d[0];
            s += e;
        }
    }
    s
}
