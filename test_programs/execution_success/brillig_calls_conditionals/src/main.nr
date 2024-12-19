// Tests a very simple program.
//
// The features being tested is brillig calls with conditionals
fn main(x: [u32; 3]) {
    unsafe {
        //@safety: testing context
        assert(entry_point(x[0]) == 7);
        assert(entry_point(x[1]) == 8);
        assert(entry_point(x[2]) == 9);
        assert(entry_point(42) == 0);
    }
}

unconstrained fn inner_1() -> u32 {
    7
}

unconstrained fn inner_2() -> u32 {
    8
}

unconstrained fn inner_3() -> u32 {
    9
}

unconstrained fn entry_point(x: u32) -> u32 {
    let mut result: u32 = 0;

    if x == 1 {
        result = inner_1();
    } else if x == 2 {
        result = inner_2();
    } else if x == 3 {
        result = inner_3();
    }

    result
}

