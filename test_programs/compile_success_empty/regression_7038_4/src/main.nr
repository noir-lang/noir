// This program is a reduction of regression_7038_3 that led to a monomorphizer crash
trait BigNumTrait {}

trait CurveParamsTrait<BigNum> {
    fn one()
    where
        BigNum: BigNumTrait;
}

pub struct MyBigNum;

impl BigNumTrait for MyBigNum {}

pub struct Params;
impl CurveParamsTrait<MyBigNum> for Params {

    fn one() {}
}

fn foo<C>()
where
    C: CurveParamsTrait<MyBigNum>,
{
    let _ = C::one();
}

fn main() {
    foo::<Params>();
}
