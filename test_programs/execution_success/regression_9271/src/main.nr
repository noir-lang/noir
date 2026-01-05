fn main(b: bool) -> pub bool {
    let s: [(u128, bool, [u32; 2])] = #[(0, true, [1, 2]), (3, false, [4, 5])];
    println(s);

    // safety: test
    unsafe {
        // This should print `(#[1, 0], true, #[100])`, not `(#[1, 0], false, #[1])`
        func_5((if b { #[1, 0] } else { #[0, 0, 0, 0] }, true, #[100]))
    }
}
fn func_5(a: ([u1], bool, [u32])) -> bool {
    println(a);
    a.1
}
