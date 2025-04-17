fn main() {
    // Safety: testing context
    unsafe {
        foo(bar);
    }
}

unconstrained fn foo<Env>(f: fn[Env]()) {
    f();
    f();
}

fn bar() {
    bar();
    bar();
}
