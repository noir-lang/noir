fn main() {
    // Safety: testing context
    unsafe {
        foo(bar);
    }
}

unconstrained fn foo<Env>(f: unconstrained fn[Env]()) {
    f();
    f();
}

unconstrained fn bar() {
    bar();
    bar();
}
