// This is the same as `lambda_taking_lambda_regression_8543` except that there is
// actually a variant which has been constructed.
unconstrained fn main() -> pub i64 {
    func_3(func_2)
}

unconstrained fn func_2(a: unconstrained fn(u128) -> bool) -> bool {
    a(1)
}
unconstrained fn func_3(b: unconstrained fn(unconstrained fn(u128) -> bool) -> bool) -> i64 {
    let _s = b(func_1);
    0
}

unconstrained fn func_1(a: u128) -> bool {
    a > 0
}
