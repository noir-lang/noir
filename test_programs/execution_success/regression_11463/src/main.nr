// Regression test for https://github.com/noir-lang/noir/issues/11463
//
// The bug was that vector operations like push_front were incorrectly
// considered "pure" for the purpose of clone skipping. This caused the
// inc_rc instruction to be omitted, allowing Brillig's copy-on-write
// optimization to corrupt arrays that were still in use.
//
// In the recursive function below, `c` is passed to push_front and then
// used again in the assertion after the recursive call returns. Without
// proper reference counting, `c` would contain corrupted data.

fn main() {
    let mut n: u32 = 2;
    // Safety: testing unconstrained execution
    unsafe { entry(n) }
}

unconstrained fn recursive(mut c: [Field], n: &mut u32, expected: [Field]) {
    if *n != 0 {
        *n -= 1;
        // After this recursive call returns, `c` should still have its original values.
        // The bug was that push_front would reuse c's memory when RC=1, corrupting c.
        recursive(c.push_front(0), n, expected.push_front(0));

        // This assertion would fail before the fix because c was corrupted
        assert(c == expected, "slice was corrupted after recursive call");
    }
}

unconstrained fn entry(mut n: u32) {
    let initial: [Field] = &[1, 2];
    recursive(initial, &mut n, initial);
}
