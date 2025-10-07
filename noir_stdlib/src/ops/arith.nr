use crate::convert::AsPrimitive;

// docs:start:add-trait
pub trait Add {
    fn add(self, other: Self) -> Self;
}
// docs:end:add-trait

impl Add for Field {
    fn add(self, other: Field) -> Field {
        self + other
    }
}

impl Add for u128 {
    fn add(self, other: u128) -> u128 {
        self + other
    }
}
impl Add for u64 {
    fn add(self, other: u64) -> u64 {
        self + other
    }
}
impl Add for u32 {
    fn add(self, other: u32) -> u32 {
        self + other
    }
}
impl Add for u16 {
    fn add(self, other: u16) -> u16 {
        self + other
    }
}
impl Add for u8 {
    fn add(self, other: u8) -> u8 {
        self + other
    }
}
impl Add for u1 {
    fn add(self, other: u1) -> u1 {
        self + other
    }
}

impl Add for i8 {
    fn add(self, other: i8) -> i8 {
        self + other
    }
}
impl Add for i16 {
    fn add(self, other: i16) -> i16 {
        self + other
    }
}
impl Add for i32 {
    fn add(self, other: i32) -> i32 {
        self + other
    }
}
impl Add for i64 {
    fn add(self, other: i64) -> i64 {
        self + other
    }
}

// docs:start:sub-trait
pub trait Sub {
    fn sub(self, other: Self) -> Self;
}
// docs:end:sub-trait

impl Sub for Field {
    fn sub(self, other: Field) -> Field {
        self - other
    }
}

impl Sub for u128 {
    fn sub(self, other: u128) -> u128 {
        self - other
    }
}
impl Sub for u64 {
    fn sub(self, other: u64) -> u64 {
        self - other
    }
}
impl Sub for u32 {
    fn sub(self, other: u32) -> u32 {
        self - other
    }
}
impl Sub for u16 {
    fn sub(self, other: u16) -> u16 {
        self - other
    }
}
impl Sub for u8 {
    fn sub(self, other: u8) -> u8 {
        self - other
    }
}
impl Sub for u1 {
    fn sub(self, other: u1) -> u1 {
        self - other
    }
}

impl Sub for i8 {
    fn sub(self, other: i8) -> i8 {
        self - other
    }
}
impl Sub for i16 {
    fn sub(self, other: i16) -> i16 {
        self - other
    }
}
impl Sub for i32 {
    fn sub(self, other: i32) -> i32 {
        self - other
    }
}
impl Sub for i64 {
    fn sub(self, other: i64) -> i64 {
        self - other
    }
}

// docs:start:mul-trait
pub trait Mul {
    fn mul(self, other: Self) -> Self;
}
// docs:end:mul-trait

impl Mul for Field {
    fn mul(self, other: Field) -> Field {
        self * other
    }
}

impl Mul for u128 {
    fn mul(self, other: u128) -> u128 {
        self * other
    }
}
impl Mul for u64 {
    fn mul(self, other: u64) -> u64 {
        self * other
    }
}
impl Mul for u32 {
    fn mul(self, other: u32) -> u32 {
        self * other
    }
}
impl Mul for u16 {
    fn mul(self, other: u16) -> u16 {
        self * other
    }
}
impl Mul for u8 {
    fn mul(self, other: u8) -> u8 {
        self * other
    }
}
impl Mul for u1 {
    fn mul(self, other: u1) -> u1 {
        self * other
    }
}

impl Mul for i8 {
    fn mul(self, other: i8) -> i8 {
        self * other
    }
}
impl Mul for i16 {
    fn mul(self, other: i16) -> i16 {
        self * other
    }
}
impl Mul for i32 {
    fn mul(self, other: i32) -> i32 {
        self * other
    }
}
impl Mul for i64 {
    fn mul(self, other: i64) -> i64 {
        self * other
    }
}

// docs:start:div-trait
pub trait Div {
    fn div(self, other: Self) -> Self;
}
// docs:end:div-trait

impl Div for Field {
    fn div(self, other: Field) -> Field {
        self / other
    }
}

impl Div for u128 {
    fn div(self, other: u128) -> u128 {
        self / other
    }
}
impl Div for u64 {
    fn div(self, other: u64) -> u64 {
        self / other
    }
}
impl Div for u32 {
    fn div(self, other: u32) -> u32 {
        self / other
    }
}
impl Div for u16 {
    fn div(self, other: u16) -> u16 {
        self / other
    }
}
impl Div for u8 {
    fn div(self, other: u8) -> u8 {
        self / other
    }
}
impl Div for u1 {
    fn div(self, other: u1) -> u1 {
        self / other
    }
}

impl Div for i8 {
    fn div(self, other: i8) -> i8 {
        self / other
    }
}
impl Div for i16 {
    fn div(self, other: i16) -> i16 {
        self / other
    }
}
impl Div for i32 {
    fn div(self, other: i32) -> i32 {
        self / other
    }
}
impl Div for i64 {
    fn div(self, other: i64) -> i64 {
        self / other
    }
}

// docs:start:rem-trait
pub trait Rem {
    fn rem(self, other: Self) -> Self;
}
// docs:end:rem-trait

impl Rem for u128 {
    fn rem(self, other: u128) -> u128 {
        self % other
    }
}
impl Rem for u64 {
    fn rem(self, other: u64) -> u64 {
        self % other
    }
}
impl Rem for u32 {
    fn rem(self, other: u32) -> u32 {
        self % other
    }
}
impl Rem for u16 {
    fn rem(self, other: u16) -> u16 {
        self % other
    }
}
impl Rem for u8 {
    fn rem(self, other: u8) -> u8 {
        self % other
    }
}
impl Rem for u1 {
    fn rem(self, other: u1) -> u1 {
        self % other
    }
}

impl Rem for i8 {
    fn rem(self, other: i8) -> i8 {
        self % other
    }
}
impl Rem for i16 {
    fn rem(self, other: i16) -> i16 {
        self % other
    }
}
impl Rem for i32 {
    fn rem(self, other: i32) -> i32 {
        self % other
    }
}
impl Rem for i64 {
    fn rem(self, other: i64) -> i64 {
        self % other
    }
}

// docs:start:neg-trait
pub trait Neg {
    fn neg(self) -> Self;
}
// docs:end:neg-trait

// docs:start:neg-trait-impls
impl Neg for Field {
    fn neg(self) -> Field {
        -self
    }
}

impl Neg for i8 {
    fn neg(self) -> i8 {
        -self
    }
}
impl Neg for i16 {
    fn neg(self) -> i16 {
        -self
    }
}
impl Neg for i32 {
    fn neg(self) -> i32 {
        -self
    }
}
impl Neg for i64 {
    fn neg(self) -> i64 {
        -self
    }
}
// docs:end:neg-trait-impls

// docs:start:wrapping-add-trait
pub trait WrappingAdd {
    fn wrapping_add(self, y: Self) -> Self;
}
// docs:end:wrapping-add-trait

impl WrappingAdd for u1 {
    fn wrapping_add(self: u1, y: u1) -> u1 {
        self ^ y
    }
}

impl WrappingAdd for u8 {
    fn wrapping_add(self: u8, y: u8) -> u8 {
        wrapping_add_hlp(self, y)
    }
}

impl WrappingAdd for u16 {
    fn wrapping_add(self: u16, y: u16) -> u16 {
        wrapping_add_hlp(self, y)
    }
}

impl WrappingAdd for u32 {
    fn wrapping_add(self: u32, y: u32) -> u32 {
        wrapping_add_hlp(self, y)
    }
}

impl WrappingAdd for u64 {
    fn wrapping_add(self: u64, y: u64) -> u64 {
        wrapping_add_hlp(self, y)
    }
}

impl WrappingAdd for u128 {
    fn wrapping_add(self: u128, y: u128) -> u128 {
        wrapping_add_hlp(self, y)
    }
}

impl WrappingAdd for i8 {
    fn wrapping_add(self: i8, y: i8) -> i8 {
        let x = self as u8;
        x.wrapping_add(y as u8) as i8
    }
}

impl WrappingAdd for i16 {
    fn wrapping_add(self: i16, y: i16) -> i16 {
        let x = self as u16;
        x.wrapping_add(y as u16) as i16
    }
}

impl WrappingAdd for i32 {
    fn wrapping_add(self: i32, y: i32) -> i32 {
        let x = self as u32;
        x.wrapping_add(y as u32) as i32
    }
}

impl WrappingAdd for i64 {
    fn wrapping_add(self: i64, y: i64) -> i64 {
        let x = self as u64;
        x.wrapping_add(y as u64) as i64
    }
}
impl WrappingAdd for Field {
    fn wrapping_add(self: Field, y: Field) -> Field {
        self + y
    }
}

// docs:start:wrapping-sub-trait
pub trait WrappingSub {
    fn wrapping_sub(self, y: Self) -> Self;
}
// docs:start:wrapping-sub-trait

impl WrappingSub for u1 {
    fn wrapping_sub(self: u1, y: u1) -> u1 {
        self ^ y
    }
}

impl WrappingSub for u8 {
    fn wrapping_sub(self: u8, y: u8) -> u8 {
        wrapping_sub_hlp(self, y) as u8
    }
}

impl WrappingSub for u16 {
    fn wrapping_sub(self: u16, y: u16) -> u16 {
        wrapping_sub_hlp(self, y) as u16
    }
}

impl WrappingSub for u32 {
    fn wrapping_sub(self: u32, y: u32) -> u32 {
        wrapping_sub_hlp(self, y) as u32
    }
}
impl WrappingSub for u64 {
    fn wrapping_sub(self: u64, y: u64) -> u64 {
        wrapping_sub_hlp(self, y) as u64
    }
}
impl WrappingSub for u128 {
    fn wrapping_sub(self: u128, y: u128) -> u128 {
        wrapping_sub_hlp(self, y) as u128
    }
}

impl WrappingSub for i8 {
    fn wrapping_sub(self: i8, y: i8) -> i8 {
        let x = self as u8;
        x.wrapping_sub(y as u8) as i8
    }
}

impl WrappingSub for i16 {
    fn wrapping_sub(self: i16, y: i16) -> i16 {
        let x = self as u16;
        x.wrapping_sub(y as u16) as i16
    }
}

impl WrappingSub for i32 {
    fn wrapping_sub(self: i32, y: i32) -> i32 {
        let x = self as u32;
        x.wrapping_sub(y as u32) as i32
    }
}
impl WrappingSub for i64 {
    fn wrapping_sub(self: i64, y: i64) -> i64 {
        let x = self as u64;
        x.wrapping_sub(y as u64) as i64
    }
}
impl WrappingSub for Field {
    fn wrapping_sub(self: Field, y: Field) -> Field {
        self - y
    }
}

// docs:start:wrapping-mul-trait
pub trait WrappingMul {
    fn wrapping_mul(self, y: Self) -> Self;
}
// docs:start:wrapping-mul-trait

impl WrappingMul for u1 {
    fn wrapping_mul(self: u1, y: u1) -> u1 {
        self & y
    }
}

impl WrappingMul for u8 {
    fn wrapping_mul(self: u8, y: u8) -> u8 {
        wrapping_mul_hlp(self, y)
    }
}

impl WrappingMul for u16 {
    fn wrapping_mul(self: u16, y: u16) -> u16 {
        wrapping_mul_hlp(self, y)
    }
}

impl WrappingMul for u32 {
    fn wrapping_mul(self: u32, y: u32) -> u32 {
        wrapping_mul_hlp(self, y)
    }
}
impl WrappingMul for u64 {
    fn wrapping_mul(self: u64, y: u64) -> u64 {
        wrapping_mul_hlp(self, y)
    }
}

impl WrappingMul for i8 {
    fn wrapping_mul(self: i8, y: i8) -> i8 {
        let x = self as u8;
        x.wrapping_mul(y as u8) as i8
    }
}

impl WrappingMul for i16 {
    fn wrapping_mul(self: i16, y: i16) -> i16 {
        let x = self as u16;
        x.wrapping_mul(y as u16) as i16
    }
}

impl WrappingMul for i32 {
    fn wrapping_mul(self: i32, y: i32) -> i32 {
        let x = self as u32;
        x.wrapping_mul(y as u32) as i32
    }
}

impl WrappingMul for i64 {
    fn wrapping_mul(self: i64, y: i64) -> i64 {
        let x = self as u64;
        x.wrapping_mul(y as u64) as i64
    }
}

impl WrappingMul for u128 {
    fn wrapping_mul(self: u128, y: u128) -> u128 {
        wrapping_mul128_hlp(self, y)
    }
}
impl WrappingMul for Field {
    fn wrapping_mul(self: Field, y: Field) -> Field {
        self * y
    }
}

fn wrapping_add_hlp<T>(x: T, y: T) -> T
where
    T: AsPrimitive<Field>,
    Field: AsPrimitive<T>,
{
    AsPrimitive::as_(x.as_() + y.as_())
}

fn wrapping_sub_hlp<T>(x: T, y: T) -> Field
where
    T: AsPrimitive<Field>,
{
    //340282366920938463463374607431768211456 is 2^128, it is used to avoid underflow
    x.as_() + 340282366920938463463374607431768211456 - y.as_()
}

fn wrapping_mul_hlp<T>(x: T, y: T) -> T
where
    T: AsPrimitive<Field>,
    Field: AsPrimitive<T>,
{
    AsPrimitive::as_(x.as_() * y.as_())
}

global two_pow_64: u128 = 0x10000000000000000;
/// Splits a 128 bits number into two 64 bits limbs
unconstrained fn split64(x: u128) -> (u64, u64) {
    let lo = x as u64;
    let hi = (x / two_pow_64) as u64;
    (lo, hi)
}

/// Split a 128 bits number into two 64 bits limbs
/// It will fail if the number is more than 128 bits
fn split_into_64_bit_limbs(x: u128) -> (u64, u64) {
    // Safety: the limbs are constrained below
    let (x_lo, x_hi) = unsafe { split64(x) };
    assert(x as Field == x_lo as Field + x_hi as Field * two_pow_64 as Field);
    (x_lo, x_hi)
}

#[field(bn254)]
fn wrapping_mul128_hlp(x: u128, y: u128) -> u128 {
    let (x_lo, x_hi) = split_into_64_bit_limbs(x);
    let (y_lo, y_hi) = split_into_64_bit_limbs(y);
    // Multiplication using the limbs:(x_lo + 2**64*x_hi)*(y_lo + 2**64*y_hi)=x_lo*y_lo+...
    // and skipping the terms over 2**128
    // Working with u64 limbs ensures that we cannot overflow the field modulus.
    let low = x_lo as Field * y_lo as Field;
    let lo = low as u64 as Field;
    let carry = (low - lo) / two_pow_64 as Field;
    let high = x_lo as Field * y_hi as Field + x_hi as Field * y_lo as Field + carry;
    let hi = high as u64 as Field;
    (lo + two_pow_64 as Field * hi) as u128
}

mod tests {
    #[test(should_fail_with = "custom message")]
    fn test_static_assert_custom_message() {
        crate::static_assert(1 == 2, "custom message");
    }

    mod arithmetic {
        use crate::ops::arith::{Add, Div, Mul, Neg, Rem, Sub};
        #[test]
        fn test_basic_arithmetic_traits() {
            // add
            assert_eq(5.add(3), 8);
            assert_eq(0u8.add(255u8), 255u8);
            assert_eq(42.add(58), 100);

            // sub
            assert_eq(10.sub(3), 7);
            assert_eq(100.sub(42), 58);

            // mul
            assert_eq(6.mul(7), 42);

            // div
            assert_eq(15.div(3), 5);
            assert_eq(10u8.div(3u8), 3u8);
            assert_eq(15.div(3), 5);

            // rem
            assert_eq(17.rem(5), 2);
            assert_eq(10u8.rem(3u8), 1u8);

            // neg
            assert_eq(42.neg(), -42);
            assert_eq((-10).neg(), 10);
            assert_eq(42.neg(), -42);
        }

        #[test]
        fn test_division() {
            // test division by one
            assert_eq(42.div(1), 42);
            assert_eq(0.div(1), 0);
            assert_eq(255u8.div(1u8), 255u8);

            // test division by self
            assert_eq(42.div(42), 1);
            assert_eq(1.div(1), 1);

            // test remainder
            assert_eq(42.rem(42), 0);
            assert_eq(0.rem(42), 0);
            assert_eq(1.rem(42), 1);
        }

        #[test(should_fail)]
        fn test_u8_sub_overflow_failure() {
            let _ = 0u8.sub(1u8);
        }

        #[test(should_fail)]
        fn test_u8_add_overflow_failure() {
            let _ = 255u8.add(1u8);
        }

        #[test(should_fail)]
        fn test_u8_mul_overflow_failure() {
            let _ = 255u8.mul(2u8);
        }

        #[test(should_fail)]
        fn test_u16_sub_overflow_failure() {
            let _ = 0u16.sub(1u16);
        }

        #[test(should_fail)]
        fn test_u16_add_overflow_failure() {
            let _ = 65535u16.add(1u16);
        }

        #[test(should_fail)]
        fn test_u16_mul_overflow_failure() {
            let _ = 65535u16.mul(2u16);
        }

        #[test(should_fail)]
        fn test_signed_sub_overflow_failure() {
            let val: i8 = -128;
            let _ = val.sub(1i8);
        }

        #[test(should_fail)]
        fn test_signed_overflow_failure() {
            let _ = 127i8.add(1i8);
        }

        #[test]
        fn test_field() {
            let zero: Field = 0;
            let one: Field = 1;

            // test Field basic operations
            assert_eq(zero.add(one), one);
            assert_eq(one.add(zero), one);
            assert_eq(one.sub(one), zero);
            assert_eq(one.mul(one), one);
            assert_eq(one.div(one), one);
            assert_eq(zero.neg(), zero);
            assert_eq(one.neg(), -one);
        }

    }

    mod wrapping_arithmetic {
        use crate::ops::arith::{Add, Div, Mul, Neg, Sub, WrappingAdd, WrappingMul, WrappingSub};
        #[test]
        fn test_wrapping_add() {
            assert_eq(255u8.wrapping_add(1u8), 0u8);
            assert_eq(255u8.wrapping_add(255u8), 254u8);
            assert_eq(0u8.wrapping_add(0u8), 0u8);
            assert_eq(128u8.wrapping_add(128u8), 0u8);

            // test u16 wrapping add
            assert_eq(65535u16.wrapping_add(1u16), 0u16);
            assert_eq(65535u16.wrapping_add(65535u16), 65534u16);

            // test u32 wrapping add
            assert_eq(0xffffffffu32.wrapping_add(1u32), 0u32);
            assert_eq(0xffffffffu32.wrapping_add(0xffffffffu32), 0xfffffffeu32);

            // test u64 wrapping add
            assert_eq(0xffffffffffffffffu64.wrapping_add(1u64), 0u64);
            assert_eq(
                0xffffffffffffffffu64.wrapping_add(0xffffffffffffffffu64),
                0xfffffffffffffffeu64,
            );

            // test u128 wrapping add
            assert_eq(0xffffffffffffffffffffffffffffffffu128.wrapping_add(1u128), 0u128);

            // test signed types
            assert_eq(127i8.wrapping_add(1i8), -128i8);
            let val: i8 = -128;
            assert_eq(val.wrapping_add(-1i8), 127i8);

            // test Field wrapping add
            let forty_two: Field = 42;
            let fifty_eight: Field = 58;
            let hundred: Field = 100;
            let neg_two: Field = -2;
            let two: Field = 2;
            let zero: Field = 0;
            let neg_two_hundred: Field = -200;
            let neg_one_ninety_eight: Field = -198;
            assert_eq(forty_two.wrapping_add(fifty_eight), hundred);
            assert_eq(neg_two.wrapping_add(two), zero);
            assert_eq(neg_two_hundred.wrapping_add(two), neg_one_ninety_eight);
        }

        #[test]
        fn test_wrapping_sub() {
            assert_eq(0u8.wrapping_sub(1u8), 255u8);
            assert_eq(255u8.wrapping_sub(255u8), 0u8);
            assert_eq(0u8.wrapping_sub(0u8), 0u8);
            assert_eq(1u8.wrapping_sub(2u8), 255u8);

            // test u16 wrapping sub
            assert_eq(0u16.wrapping_sub(1u16), 65535u16);
            assert_eq(65535u16.wrapping_sub(65535u16), 0u16);

            // test u32 wrapping sub
            assert_eq(0u32.wrapping_sub(1u32), 0xffffffffu32);
            assert_eq(0xffffffffu32.wrapping_sub(0xffffffffu32), 0u32);

            // test u64 wrapping sub
            assert_eq(0u64.wrapping_sub(1u64), 0xffffffffffffffffu64);
            assert_eq(0xffffffffffffffffu64.wrapping_sub(0xffffffffffffffffu64), 0u64);

            // test u128 wrapping sub
            assert_eq(0u128.wrapping_sub(1u128), 0xffffffffffffffffffffffffffffffffu128);

            // test signed types
            let val: i8 = -128;
            assert_eq(val.wrapping_sub(1i8), 127i8);
            assert_eq(127i8.wrapping_sub(-1i8), -128i8);

            // test Field wrapping sub
            let forty_two: Field = 42;
            let fifty_eight: Field = 58;
            let neg_sixteen: Field = -16;
            assert_eq(forty_two.wrapping_sub(fifty_eight), neg_sixteen);
        }

        #[test]
        fn test_wrapping_mul() {
            let zero: u128 = 0;
            let one: u128 = 1;
            let two_pow_64: u128 = 0x10000000000000000;
            let u128_max: u128 = 0xffffffffffffffffffffffffffffffff;

            assert_eq(zero, zero.wrapping_mul(one));
            assert_eq(zero, one.wrapping_mul(zero));
            assert_eq(one, one.wrapping_mul(one));
            assert_eq(zero, zero.wrapping_mul(two_pow_64));
            assert_eq(zero, two_pow_64.wrapping_mul(zero));
            assert_eq(two_pow_64, two_pow_64.wrapping_mul(one));
            assert_eq(two_pow_64, one.wrapping_mul(two_pow_64));
            assert_eq(zero, two_pow_64.wrapping_mul(two_pow_64));
            assert_eq(one, u128_max.wrapping_mul(u128_max));

            // test u8 wrapping mul
            assert_eq(255u8.wrapping_mul(2u8), 254u8);
            assert_eq(255u8.wrapping_mul(255u8), 1u8);
            assert_eq(128u8.wrapping_mul(2u8), 0u8);

            // test u16 wrapping mul
            assert_eq(65535u16.wrapping_mul(2u16), 65534u16);
            assert_eq(65535u16.wrapping_mul(65535u16), 1u16);

            // test u32 wrapping mul
            assert_eq(0xffffffffu32.wrapping_mul(2u32), 0xfffffffeu32);
            assert_eq(0xffffffffu32.wrapping_mul(0xffffffffu32), 1u32);

            // test u64 wrapping mul
            // 0xffffffffffffffffu64 is 2^64 - 1
            assert_eq(0xffffffffffffffffu64.wrapping_mul(2u64), 0xfffffffffffffffeu64);
            assert_eq(0xffffffffffffffffu64.wrapping_mul(0xffffffffffffffffu64), 1u64);

            // test signed types
            assert_eq(127i8.wrapping_mul(2i8), -2i8);
            let val: i8 = -128;
            assert_eq(val.wrapping_mul(-1i8), -128i8);

            // test Field wrapping mul
            let six: Field = 6;
            let seven: Field = 7;
            let forty_two: Field = 42;
            let neg_two: Field = -2;
            let two: Field = 2;
            let neg_four: Field = -4;
            assert_eq(six.wrapping_mul(seven), forty_two);
            assert_eq(neg_two.wrapping_mul(two), neg_four);
        }

        #[test]
        fn test_u1_behavior() {
            // u1 wrapping add is XOR
            assert_eq(0u1.wrapping_add(0u1), 0u1);
            assert_eq(0u1.wrapping_add(1u1), 1u1);
            assert_eq(1u1.wrapping_add(0u1), 1u1);
            assert_eq(1u1.wrapping_add(1u1), 0u1);

            // u1 wrapping sub is XOR
            assert_eq(0u1.wrapping_sub(0u1), 0u1);
            assert_eq(0u1.wrapping_sub(1u1), 1u1);
            assert_eq(1u1.wrapping_sub(0u1), 1u1);
            assert_eq(1u1.wrapping_sub(1u1), 0u1);

            // u1 wrapping mul is AND
            assert_eq(0u1.wrapping_mul(0u1), 0u1);
            assert_eq(0u1.wrapping_mul(1u1), 0u1);
            assert_eq(1u1.wrapping_mul(0u1), 0u1);
            assert_eq(1u1.wrapping_mul(1u1), 1u1);
        }

        // test wrapping operations is the same as the regular operations
        #[test]
        fn test_wrapping_vs_regular() {
            let u64_large = 0x123456789abcdef0u64;
            let u128_large = 0x123456789abcdef0123456789abcdef0u128;

            assert_eq(u64_large.wrapping_add(1u64), u64_large + 1u64);
            assert_eq(u64_large.wrapping_sub(1u64), u64_large - 1u64);
            assert_eq(u64_large.wrapping_mul(2u64), u64_large * 2u64);

            assert_eq(u128_large.wrapping_add(1u128), u128_large + 1u128);
            assert_eq(u128_large.wrapping_sub(1u128), u128_large - 1u128);
            assert_eq(u128_large.wrapping_mul(2u128), u128_large * 2u128);
        }

        #[test]
        fn test_field_wrapping_operations() {
            let zero: Field = 0;
            let one: Field = 1;
            let large_val = 0xffffffffffffffff;

            // test Field wrapping operations
            assert_eq(zero.wrapping_add(one), one);
            assert_eq(one.wrapping_add(large_val), one + large_val);
            assert_eq(zero.wrapping_sub(one), -one);
            assert_eq(one.wrapping_sub(large_val), one - large_val);
            assert_eq(zero.wrapping_mul(one), zero);
            assert_eq(one.wrapping_mul(large_val), large_val);

            // test Field basic operations
            assert_eq(zero.add(one), one);
            assert_eq(one.add(zero), one);
            assert_eq(one.sub(one), zero);
            assert_eq(one.mul(one), one);
            assert_eq(one.div(one), one);
            assert_eq(zero.neg(), zero);
            assert_eq(one.neg(), -one);
        }

    }

    mod split_functions {

        use crate::ops::arith::{split64, split_into_64_bit_limbs};

        // test split64 and split_into_64_bit_limbs functions
        #[test]
        fn test_split_functions() {
            let small_val = 0x123456789abcdefu128;
            let large_val = 0x123456789abcdef0123456789abcdef0u128;
            let max_val = 0xffffffffffffffffffffffffffffffffu128;

            // test split64 (unconstrained)
            // Safety: testing
            unsafe {
                let (lo, hi) = split64(small_val);
                assert_eq(lo, 0x123456789abcdefu64);
                assert_eq(hi, 0u64);

                let (lo2, hi2) = split64(large_val);
                assert_eq(lo2, 0x123456789abcdef0u64);
                assert_eq(hi2, 0x123456789abcdef0u64);
            }

            // test split_into_64_bit_limbs (constrained)
            let (lo3, hi3) = split_into_64_bit_limbs(small_val);
            assert_eq(lo3, 0x123456789abcdefu64);
            assert_eq(hi3, 0u64);

            let (lo4, hi4) = split_into_64_bit_limbs(large_val);
            assert_eq(lo4, 0x123456789abcdef0u64);
            assert_eq(hi4, 0x123456789abcdef0u64);

            let (lo5, hi5) = split_into_64_bit_limbs(max_val);
            assert_eq(lo5, 0xffffffffffffffffu64);
            assert_eq(hi5, 0xffffffffffffffffu64);
        }
    }
}
