// Regression: an oracle must not accept a function-typed parameter. Previously this passed
// the frontend, then defunctionalization lowered the function value to numeric tags, leaving
// the foreign call's arity inconsistent with the declared oracle signature.
fn main(x: i32) -> pub i32 {
    // Safety: test reproduction
    let res = unsafe { call_oracle(f) };
    assert(res == x);
    x
}

#[oracle(my_oracle)]
unconstrained fn my_oracle(f: fn(i32) -> i32) -> i32 {}

unconstrained fn call_oracle(g: fn(i32) -> i32) -> i32 {
    my_oracle(g)
}

fn f(_: i32) -> i32 {
    1
}
