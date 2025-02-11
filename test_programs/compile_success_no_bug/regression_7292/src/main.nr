unconstrained fn foo<T>(_x: T) -> Field {
    0
}

fn main() {
    /// Safety: test
    unsafe {
        assert(foo(0) == 0);
    }
}
