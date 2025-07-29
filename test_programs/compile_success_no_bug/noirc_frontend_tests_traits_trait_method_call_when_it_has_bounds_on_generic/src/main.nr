
    trait BigNum {}

    trait BigCurve<B>
    where
        B: BigNum,
    {
        fn new() -> Self;
    }

    pub fn foo<B: BigNum, Curve: BigCurve<B>>() {
        let _: Curve = BigCurve::new();
    }

    fn main() {}
    