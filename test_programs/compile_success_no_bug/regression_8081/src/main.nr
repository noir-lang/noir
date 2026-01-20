fn main() {
    // safety: Compilation should succeed without Rust stack overflow during constant folding.
    unsafe {
        foo(bar, true);
    }
}

unconstrained fn foo<Env>(f: fn[Env](bool), recur: bool) {
    f(recur);
}

fn bar(recur: bool) {
    if recur {
        bar(recur);
    }
}
