trait BigNumTrait {}

pub struct MyBigNum;

impl crate::BigNumTrait for MyBigNum {}

trait CurveParamsTrait<BigNum> {
    // The difference between this and regression_7038 and regression_7038_2 is that
    // here the where clause is on the method, not the trait
    fn one()
    where
        BigNum: BigNumTrait;
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
