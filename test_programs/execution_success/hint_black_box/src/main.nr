use std::hint::black_box;

fn main(a: u32, b: u32) {
    // This version unrolls into a number of additions
    assert_eq(loop(5, a), b);
    // This version simplifies into a single `constraint 50 == b`
    assert_eq(loop(5, 10), b);
    // This version should not simplify down to a single constraint,
    // it should treat 10 as opaque:
    assert_eq(loop(5, black_box(10)), b);

    // Check array handling.
    let arr = [a, a, a, a, a];

    assert_eq(array_sum(arr), b);
    assert_eq(array_sum(black_box(arr)), b);

    assert_eq(slice_sum(arr.as_slice()), b);
    assert_eq(slice_sum(black_box(arr).as_slice()), b);

    // This doesn't work because by calling `black_box` on a slice the compiler
    // loses track of the length, and then cannot unroll the loop for ACIR.
    //assert_eq(slice_sum(black_box(arr.as_slice())), b);

    // But we can pass a blackboxed slice to Brillig.
    let s = unsafe {
        //@safety: testing context
        brillig_slice_sum(black_box(arr.as_slice()))
    };
    assert_eq(s, b);

    let mut d = b;
    // This gets completely eliminated:
    let mut c = 0;
    set_ref(&mut c, &mut d);
    assert_eq(c, b);

    // This way the constraint is preserved:
    let mut c = 0;
    set_ref(&mut c, &mut black_box(d));
    assert_eq(c, b);

    // A reference over the output of black box is not the original variable:
    let mut c = 0;
    set_ref(&mut black_box(c), &mut d);
    assert_eq(c, 0);

    // This would cause a causes a crash during SSA passes unless it's a Brillig runtime:
    //  > Could not resolve some references to the array. All references must be resolved at compile time
    // The SSA cannot have Allocate by the time we start generating ACIR, but `black_box` prevents them
    // from being optimised away during SSA passes.
    // If we use `--force-brillig` then the it doesn't crash but the assertion fails because `mem2reg`
    // eliminates the storing to the reference.
    //let mut c = 0;
    //set_ref(black_box(&mut c), black_box(&mut d));
    //assert_eq(c, b);
}

fn loop(n: u32, k: u32) -> u32 {
    let mut sum = 0;
    for _ in 0..n {
        sum = sum + k;
    }
    sum
}

fn array_sum<let N: u32>(xs: [u32; N]) -> u32 {
    let mut sum = 0;
    for i in 0..N {
        sum = sum + xs[i];
    }
    sum
}

fn slice_sum(xs: [u32]) -> u32 {
    let mut sum = 0;
    for x in xs {
        sum = sum + x;
    }
    sum
}

unconstrained fn brillig_slice_sum(xs: [u32]) -> u32 {
    let mut sum = 0;
    for x in xs {
        sum = sum + x;
    }
    sum
}

fn set_ref(c: &mut u32, b: &mut u32) {
    *c = *b;
}
