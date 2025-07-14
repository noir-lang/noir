pub trait BigCurve<B> {
    /// Returns the canonical generator point of the curve.
    fn one() -> Self;
}

struct Bn254 {}

impl<B> BigCurve<B> for Bn254 {
    fn one() -> Self { Bn254 {} }
}

fn main() {
    let _ = <Bn254 as BigCurve<()>>::one();
}
