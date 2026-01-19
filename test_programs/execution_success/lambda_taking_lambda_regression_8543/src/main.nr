// Regression for issue #8543 (https://github.com/noir-lang/noir/issues/8543)
unconstrained fn main() -> pub i64 {
    func_3(func_2)
}

unconstrained fn func_2(a: unconstrained fn(u128) -> bool) -> bool {
    a(1)
}
unconstrained fn func_3(_b: unconstrained fn(unconstrained fn(u128) -> bool) -> bool) -> i64 {
    0
}
