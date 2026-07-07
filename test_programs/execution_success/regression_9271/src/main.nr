fn main(b: bool) -> pub bool {
    let s: [(u128, bool, [u32; 2])] = [(0, true, [1, 2]), (3, false, [4, 5])].as_vector();
    println(s);

    // safety: test
    unsafe {
        // This should print `([true, false].as_vector(), true, [100].as_vector())`, not `([true, false].as_vector(), false, [1].as_vector())`
        func_5((
            if b {
                [true, false].as_vector()
            } else {
                [false, false, false, false].as_vector()
            }, true, [100].as_vector(),
        ))
    }
}
fn func_5(a: ([bool], bool, [u32])) -> bool {
    println(a);
    a.1
}
