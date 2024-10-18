// Tests that we run liveness in block parameters by trying to create too many block parameters to fit in the stack
// Uses up 10 stack items
struct Inner {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
    g: u64,
    h: u64,
    i: u64,
    j: u64,
}

// Uses up 50 stack items
struct Middle {
    inner_a: Inner,
    inner_b: Inner,
    inner_c: Inner,
    inner_d: Inner,
    inner_e: Inner,
}

// Uses up 500 stack items
struct Outer {
    middle_a: Middle,
    middle_b: Middle,
    middle_c: Middle,
    middle_d: Middle,
    middle_e: Middle,
    middle_f: Middle,
    middle_g: Middle,
    middle_h: Middle,
}

// If we don't take into account block parameter liveness, this function will need 5*500=2500 stack items
unconstrained fn main(conditions: [bool; 5]) -> pub Outer {
    let out0 = if conditions[0] {
        let mut outer: Outer = std::mem::zeroed();
        outer.middle_a.inner_a.a = 1;
        outer
    } else {
        let mut outer: Outer = std::mem::zeroed();
        outer.middle_f.inner_c.d = 2;
        outer
    };

    let out1 = if conditions[1] {
        let mut new_outer = out0;
        new_outer.middle_a.inner_a.b = 3;
        new_outer
    } else {
        let mut new_outer = out0;
        new_outer.middle_f.inner_c.c = 4;
        new_outer
    };

    let out2 = if conditions[2] {
        let mut new_outer = out1;
        new_outer.middle_a.inner_a.c = 5;
        new_outer
    } else {
        let mut new_outer = out1;
        new_outer.middle_f.inner_c.b = 6;
        new_outer
    };

    let out3 = if conditions[3] {
        let mut new_outer = out2;
        new_outer.middle_a.inner_a.d = 7;
        new_outer
    } else {
        let mut new_outer = out2;
        new_outer.middle_f.inner_c.a = 8;
        new_outer
    };

    let out4 = if conditions[4] {
        let mut new_outer = out3;
        new_outer.middle_a.inner_a.f = 9;
        new_outer
    } else {
        let mut new_outer = out3;
        new_outer.middle_f.inner_c.f = 10;
        new_outer
    };

    out4
}
