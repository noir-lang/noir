use bignum::bignum::BigNum;
use bignum::fields::U4096::U4096;

fn main() {}

#[test]
fn test_mul_u4096() {
    test_mul::<35, U4096>()
}

fn test_mul<let N: u32, BN>()
where
    BN: BigNum,
{
    let a: BN = BN::derive_from_seed([1, 2, 3, 4]);
    let b: BN = BN::derive_from_seed([4, 5, 6, 7]);
    let c: BN = (a + b) * (a + b);
    let d: BN = (a * a) + (b * b) + (a * b) + (b * a);
    assert(c.eq(d));
}
