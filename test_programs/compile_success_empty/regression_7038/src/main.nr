trait BigNumTrait {}

pub struct MyBigNum;

impl crate::BigNumTrait for MyBigNum {}

trait CurveParamsTrait<BigNum>
where
    BigNum: BigNumTrait,
{
    fn one();
}

pub struct BN254Params;
impl CurveParamsTrait<MyBigNum> for BN254Params {

    fn one() {}
}

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
