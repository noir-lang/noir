// Tests a simple loop where we expect loop invariant instructions
// to be hoisted to the loop's pre-header block.
global U32_MAX: u32 = 4294967295;

fn main(x: u32, y: u32) {
    simple_loop(4, x, y);
    loop_with_predicate(4, x, y);
    array_read_loop(4, x);
    //Safety: testing for failing assert under condition
    unsafe {
        loop_with_break_under_predicate(4, y, x);
    }
}

fn simple_loop(upper_bound: u32, x: u32, y: u32) {
    for _ in 0..upper_bound {
        let mut z = x * y;
        z = z * x;
        assert_eq(z, 12);
    }
}

fn loop_with_predicate(upper_bound: u32, x: u32, y: u32) {
    for _ in 0..upper_bound {
        if x == 5 {
            let mut z = U32_MAX * y;
            assert_eq(((3 * y as i8) * 127), 12);
            assert_eq(z, 12);
        }
    }
}

fn array_read_loop(upper_bound: u32, x: u32) {
    let arr = [2; 5];
    for i in 0..upper_bound {
        for j in 0..upper_bound {
            for _ in 0..upper_bound {
                assert_eq(arr[i], x);
                assert_eq(arr[j], x);
            }
        }
    }
}

unconstrained fn loop_with_break_under_predicate(upper_bound: u32, x: u32, y: u32) {
    for _ in 0..upper_bound {
        if x == 3 {
            break;
        }
        let mut z = U32_MAX * y;
        assert_eq(z, 12);
    }
}
