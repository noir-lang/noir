// Regression for noir-lang/noir-claude#1844, `static_assert` consumer.
//
// The body's first `if` becomes the loop header's `jmpif`, but both of its arms stay inside the
// loop; the loop actually exits through `i == x`. Reading the range `[12, 10)` off the `i < 10`
// comparison would classify the loop as never executing and delete every assertion inside it.
// `i` starts at 12, so the `else` arm runs on the first iteration and the `static_assert(false)`
// must make this program fail to compile.
unconstrained fn f(x: u32) -> u32 {
    let mut i: u32 = 12;
    let mut c: u32 = 0;
    loop {
        if i < 10 {
            c += 1;
        } else {
            std::static_assert(false, "else arm ran");
        }
        if i == x {
            break;
        }
        i += 1;
    }
    c
}

fn main(x: u32) -> pub u32 {
    // Safety: regression test for unconstrained loop shape
    unsafe {
        f(x)
    }
}
