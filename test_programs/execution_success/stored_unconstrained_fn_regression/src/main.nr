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
    unsafe { f(x) }
}

fn get_choose() -> unconstrained fn(u32) -> u32 {
    choose
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
}
