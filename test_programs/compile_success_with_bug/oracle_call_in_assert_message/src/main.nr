#[oracle(foo)]
unconstrained fn foo() -> str<4> {}

fn main() {
    // Safety:
    unsafe {
        assert(true, foo());
    }
}
