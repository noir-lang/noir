use std::runtime::is_unconstrained;

struct Holder {
    f: unconstrained fn(u32) -> u32,
}

fn choose(x: u32) -> u32 {
    if is_unconstrained() {
        x + 100
    } else {
        x + 1
    }
}

fn call_unc(f: unconstrained fn(u32) -> u32, x: u32) -> u32 {
    // Safety: regression test for stored unconstrained fn dispatch
    unsafe {
        f(x)
    }
}

fn get_choose() -> unconstrained fn(u32) -> u32 {
    choose
}

/// Returns 1 when compiled as a constrained function, 100 when compiled as an unconstrained one.
fn probe() -> u32 {
    if is_unconstrained() {
        100
    } else {
        1
    }
}

fn other(x: u32) -> u32 {
    x + 200
}

fn takes_unconstrained_fn(_f: unconstrained fn(u32) -> u32, r: u32) -> u32 {
    r
}

fn pick(cond: bool) -> unconstrained fn(u32) -> u32 {
    if cond {
        choose
    } else {
        choose
    }
}

fn block_return() -> unconstrained fn(u32) -> u32 {
    let _x = 1;
    choose
}

fn main() {
    // 1. let statement RHS
    let f: unconstrained fn(u32) -> u32 = choose;
    assert_eq(call_unc(f, 1), 101);

    // 2. struct constructor field
    let holder = Holder { f: choose };
    assert_eq(call_unc(holder.f, 2), 102);

    // 3. assignment RHS
    let mut holder2 = Holder { f: choose };
    holder2.f = choose;
    assert_eq(call_unc(holder2.f, 3), 103);

    // 4. function return position (simple ident)
    assert_eq(call_unc(get_choose(), 4), 104);

    // 5. if as RHS of unconstrained-fn let
    let h: unconstrained fn(u32) -> u32 = if true { choose } else { choose };
    assert_eq(call_unc(h, 5), 105);

    // 6. function return whose body is an if-expression
    assert_eq(call_unc(pick(true), 6), 106);

    // 7. function return whose body is a block with a terminal function reference
    assert_eq(call_unc(block_return(), 7), 107);

    // Cases 8-12 cover the other direction: an `unconstrained fn` target must force only the
    // value that occupies the slot. A function value nested inside the same expression that
    // carries its own constrained type, or that sits in a position not producing the slot's
    // value, keeps the runtime its own type declares.
    //
    // Each case compares against `probe()` called from `main` directly, so the expectation
    // tracks the runtime `main` itself was compiled for. That is 1 normally, and 100 under
    // `--force-brillig`, where every function in the program is legitimately unconstrained.
    let expected = probe();

    // 8. nested `let` with its own constrained `fn` type, inside a forced `let` RHS
    let mut a: u32 = 0;
    let _a: unconstrained fn(u32) -> u32 = {
        let g: fn() -> u32 = probe;
        a = g();
        choose
    };
    assert_eq(a, expected);

    // 9. unannotated lambda inside a forced `let` RHS
    let mut b: u32 = 0;
    let _b: unconstrained fn(u32) -> u32 = {
        let g = || if is_unconstrained() { 100 } else { 1 };
        b = g();
        choose
    };
    assert_eq(b, expected);

    // 10. call argument whose parameter type is `unconstrained fn`
    let mut c: u32 = 0;
    let _c = takes_unconstrained_fn(
        {
            let g: fn() -> u32 = probe;
            c = g();
            choose
        },
        0,
    );
    assert_eq(c, expected);

    // 11. struct constructor field
    let mut d: u32 = 0;
    let _d = Holder {
        f: {
            let g: fn() -> u32 = probe;
            d = g();
            choose
        },
    };
    assert_eq(d, expected);

    // 12. `if` condition -- the branches produce the slot's value, the condition does not, so a
    // lambda built in the condition matches `main`'s runtime and `choose` is selected. Forcing
    // the condition's lambda would select `other` instead, giving 208.
    let cond_probe: unconstrained fn(u32) -> u32 = if (|| probe())() == expected {
        choose
    } else {
        other
    };
    assert_eq(call_unc(cond_probe, 8), 108);
}
