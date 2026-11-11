// `step` keeps its loop state behind `&mut` parameters. After it is inlined into `run`,
// the inner loop's exit is a `jmpif` back to the outer loop header whose condition (`old`)
// is a header parameter that the same edge reassigns (to the new `flag`).
fn main(x: bool, y: u32) {
    // Safety: test
    let result = unsafe { run(x, y) };
    assert_eq(result, 106);
}

unconstrained fn run(x: bool, y: u32) -> u32 {
    let mut flag = x;
    let mut n: u32 = 0;
    step(&mut flag, &mut n, y);
    n
}

unconstrained fn step(flag: &mut bool, n: &mut u32, y: u32) {
    loop {
        let old = *flag;
        *flag = *n < y;
        *n += 1;
        if *n > 20 {
            break;
        }
        while !old {
            *n += 100;
            if *n > y {
                break;
            }
        }
    }
}
