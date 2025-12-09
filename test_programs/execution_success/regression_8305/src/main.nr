fn main(a: i32) -> pub i32 {
    let mut r: i32 = 0;
    let s: i32 = -2;
    let e: i32 = -1;
    for _ in (s - 0)..(e + 0) {
        r = r + a;
    }
    r
}
