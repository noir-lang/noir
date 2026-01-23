fn main() {
    // If we end up with the Brillig lambda being dispatched to from ACIR in SSA,
    // the build will report it as an underconstrained call and a "bug".
    let (x, f) = std::hint::black_box((1, |x| x + 1));
    println(f(x));

    // This shows that if we didn't bypass the array argument, we would still get a "bug"
    // reported because of the dispatch to the Brillig lambda.
    let fs = std::hint::black_box([(2, |x| x + 2)]);
    let (x, f) = fs[0];
    println(f(x));
}
