trait BigNumTrait {}

pub struct MyBigNum;

impl crate::BigNumTrait for MyBigNum {}

trait CurveParamsTrait<BigNum>
where
    BigNum: BigNumTrait,
{
    // The difference between this and regression_7083 is that here
    // this is a default method.
    fn one() {}
}

pub struct BN254Params;
impl CurveParamsTrait<MyBigNum> for BN254Params {}

trait BigCurveTrait {
    fn two();
}

pub struct BigCurve<BigNum, CurveParams> {}

type BN254 = BigCurve<MyBigNum, BN254Params>;

impl<BigNum, CurveParams> BigCurveTrait for BigCurve<BigNum, CurveParams>
where
    BigNum: BigNumTrait,
    CurveParams: CurveParamsTrait<BigNum>,
{
    fn two() {
        let _ = CurveParams::one();
    }
}

fn main() {
    let _ = BN254::two();
}
