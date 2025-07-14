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
