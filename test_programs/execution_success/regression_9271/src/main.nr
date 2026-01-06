fn main(b: bool) -> pub bool {
    let s: [(u128, bool, [u32; 2])] = [(0, true, [1, 2]), (3, false, [4, 5])].as_vector();
    println(s);

    // safety: test
    unsafe {
        // This should print `([1, 0].as_vector(), true, [100].as_vector())`, not `([1, 0].as_vector(), false, [1].as_vector())`
        func_5((if b { [1, 0].as_vector() } else { [0, 0, 0, 0].as_vector() }, true, [100].as_vector()))
    }
}
fn func_5(a: ([u1], bool, [u32])) -> bool {
    println(a);
    a.1
}
