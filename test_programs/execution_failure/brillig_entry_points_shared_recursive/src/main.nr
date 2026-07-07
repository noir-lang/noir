fn main() {
    baz();
}

unconstrained fn foo<Env>(f: fn[Env]()) {
    println("foo");
    bar(f);
}

unconstrained fn bar<Env>(f: fn[Env]()) {
println("bar");
    f();
}

fn baz() {
    // Safety: testing context
    unsafe {
        foo(baz);
        bar(baz);
    }
}