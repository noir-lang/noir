fn main() -> pub Field {
    // If we end up with the Brillig lambda being dispatched to from ACIR in SSA,
    // the build will report it as an underconstrained call and a bug.
    let (x, f) = std::hint::black_box((1, |x| x + 1));
    f(x)
}
