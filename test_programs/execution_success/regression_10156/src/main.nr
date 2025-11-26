pub fn main() -> pub Field {
    println(foo); // normal
    println(bar); // oracle
    println(std::as_witness); // intrinsic
    println(std::hash::blake2s::<10>); // builtin

    // safety:
    unsafe {
        unconstrained_take_bar(bar);
    }
    take_bar(bar);

    // safety:
    unsafe {
        unconstrained_take_as_witness(std::as_witness);
    }
    take_as_witness(std::as_witness);
    std::as_witness(foo());

    take_foo(foo)
}

fn foo() -> Field {
    0
}

#[oracle(barnacle)]
unconstrained fn bar() -> Field {}

fn take_foo(foo: fn() -> Field) -> Field {
    foo()
}

fn take_bar(_bar: unconstrained fn() -> Field) {}
unconstrained fn unconstrained_take_bar(_bar: unconstrained fn() -> Field) {}

fn take_as_witness(aw: fn(Field) -> ()) {
    aw(0)
}
unconstrained fn unconstrained_take_as_witness(aw: fn(Field) -> ()) {
    aw(0)
}
