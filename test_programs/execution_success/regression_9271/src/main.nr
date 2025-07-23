fn main(b: bool) -> pub bool {
    // safety: test
    unsafe {
        func_5((if b { &[0, 0] } else { &[0, 0, 0, 0] }, true, &[100]))
    }
}
fn func_5(a: ([u1], bool, [u32])) -> bool {
    println(a);
    a.1
}
