fn main() {
    let func = foo;
    expect_unconstrained(func);
}

fn foo() {}

fn expect_unconstrained(_func: unconstrained fn() -> ()) {}
