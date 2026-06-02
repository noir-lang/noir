fn main() {
    comptime {
        foo();
    }
}

unconstrained fn foo() {
    foo();
}