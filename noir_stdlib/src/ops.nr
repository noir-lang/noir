// docs:start:add-trait
trait Add {
    fn add(self, other: Self) -> Self;
}
// docs:end:add-trait

impl Add for Field { fn add(self, other: Field) -> Field { self + other } }

impl Add for u8 { fn add(self, other: u8) -> u8 { self + other } }
impl Add for u16 { fn add(self, other: u16) -> u16 { self + other } }
impl Add for u32 { fn add(self, other: u32) -> u32 { self + other } }
impl Add for u64 { fn add(self, other: u64) -> u64 { self + other } }

impl Add for i8 { fn add(self, other: i8) -> i8 { self + other } }
impl Add for i16 { fn add(self, other: i16) -> i16 { self + other } }
impl Add for i32 { fn add(self, other: i32) -> i32 { self + other } }
impl Add for i64 { fn add(self, other: i64) -> i64 { self + other } }

// docs:start:sub-trait
trait Sub {
    fn sub(self, other: Self) -> Self;
}
// docs:end:sub-trait

impl Sub for Field { fn sub(self, other: Field) -> Field { self - other } }

impl Sub for u8 { fn sub(self, other: u8) -> u8 { self - other } }
impl Sub for u16 { fn sub(self, other: u16) -> u16 { self - other } }
impl Sub for u32 { fn sub(self, other: u32) -> u32 { self - other } }
impl Sub for u64 { fn sub(self, other: u64) -> u64 { self - other } }

impl Sub for i8 { fn sub(self, other: i8) -> i8 { self - other } }
impl Sub for i16 { fn sub(self, other: i16) -> i16 { self - other } }
impl Sub for i32 { fn sub(self, other: i32) -> i32 { self - other } }
impl Sub for i64 { fn sub(self, other: i64) -> i64 { self - other } }

// docs:start:mul-trait
trait Mul {
    fn mul(self, other: Self) -> Self;
}
// docs:end:mul-trait

impl Mul for Field { fn mul(self, other: Field) -> Field { self * other } }

impl Mul for u8 { fn mul(self, other: u8) -> u8 { self * other } }
impl Mul for u16 { fn mul(self, other: u16) -> u16 { self * other } }
impl Mul for u32 { fn mul(self, other: u32) -> u32 { self * other } }
impl Mul for u64 { fn mul(self, other: u64) -> u64 { self * other } }

impl Mul for i8 { fn mul(self, other: i8) -> i8 { self * other } }
impl Mul for i16 { fn mul(self, other: i16) -> i16 { self * other } }
impl Mul for i32 { fn mul(self, other: i32) -> i32 { self * other } }
impl Mul for i64 { fn mul(self, other: i64) -> i64 { self * other } }

// docs:start:div-trait
trait Div {
    fn div(self, other: Self) -> Self;
}
// docs:end:div-trait

impl Div for Field { fn div(self, other: Field) -> Field { self / other } }

impl Div for u8 { fn div(self, other: u8) -> u8 { self / other } }
impl Div for u16 { fn div(self, other: u16) -> u16 { self / other } }
impl Div for u32 { fn div(self, other: u32) -> u32 { self / other } }
impl Div for u64 { fn div(self, other: u64) -> u64 { self / other } }

impl Div for i8 { fn div(self, other: i8) -> i8 { self / other } }
impl Div for i16 { fn div(self, other: i16) -> i16 { self / other } }
impl Div for i32 { fn div(self, other: i32) -> i32 { self / other } }
impl Div for i64 { fn div(self, other: i64) -> i64 { self / other } }

// docs:start:rem-trait
trait Rem{
    fn rem(self, other: Self) -> Self;
}
// docs:end:rem-trait

impl Rem for u8 { fn rem(self, other: u8) -> u8 { self % other } }
impl Rem for u16 { fn rem(self, other: u16) -> u16 { self % other } }
impl Rem for u32 { fn rem(self, other: u32) -> u32 { self % other } }
impl Rem for u64 { fn rem(self, other: u64) -> u64 { self % other } }

impl Rem for i8 { fn rem(self, other: i8) -> i8 { self % other } }
impl Rem for i16 { fn rem(self, other: i16) -> i16 { self % other } }
impl Rem for i32 { fn rem(self, other: i32) -> i32 { self % other } }
impl Rem for i64 { fn rem(self, other: i64) -> i64 { self % other } }

// docs:start:bitor-trait
trait BitOr {
    fn bitor(self, other: Self) -> Self;
}
// docs:end:bitor-trait

impl BitOr for bool { fn bitor(self, other: bool) -> bool { self | other } }

impl BitOr for u8 { fn bitor(self, other: u8) -> u8 { self | other } }
impl BitOr for u16 { fn bitor(self, other: u16) -> u16 { self | other } }
impl BitOr for u32 { fn bitor(self, other: u32) -> u32 { self | other } }
impl BitOr for u64 { fn bitor(self, other: u64) -> u64 { self | other } }

impl BitOr for i8 { fn bitor(self, other: i8) -> i8 { self | other } }
impl BitOr for i16 { fn bitor(self, other: i16) -> i16 { self | other } }
impl BitOr for i32 { fn bitor(self, other: i32) -> i32 { self | other } }
impl BitOr for i64 { fn bitor(self, other: i64) -> i64 { self | other } }

// docs:start:bitand-trait
trait BitAnd {
    fn bitand(self, other: Self) -> Self;
}
// docs:end:bitand-trait

impl BitAnd for bool { fn bitand(self, other: bool) -> bool { self & other } }

impl BitAnd for u8 { fn bitand(self, other: u8) -> u8 { self & other } }
impl BitAnd for u16 { fn bitand(self, other: u16) -> u16 { self & other } }
impl BitAnd for u32 { fn bitand(self, other: u32) -> u32 { self & other } }
impl BitAnd for u64 { fn bitand(self, other: u64) -> u64 { self & other } }

impl BitAnd for i8 { fn bitand(self, other: i8) -> i8 { self & other } }
impl BitAnd for i16 { fn bitand(self, other: i16) -> i16 { self & other } }
impl BitAnd for i32 { fn bitand(self, other: i32) -> i32 { self & other } }
impl BitAnd for i64 { fn bitand(self, other: i64) -> i64 { self & other } }

// docs:start:bitxor-trait
trait BitXor {
    fn bitxor(self, other: Self) -> Self;
}
// docs:end:bitxor-trait

impl BitXor for bool { fn bitxor(self, other: bool) -> bool { self ^ other } }

impl BitXor for u8 { fn bitxor(self, other: u8) -> u8 { self ^ other } }
impl BitXor for u16 { fn bitxor(self, other: u16) -> u16 { self ^ other } }
impl BitXor for u32 { fn bitxor(self, other: u32) -> u32 { self ^ other } }
impl BitXor for u64 { fn bitxor(self, other: u64) -> u64 { self ^ other } }

impl BitXor for i8 { fn bitxor(self, other: i8) -> i8 { self ^ other } }
impl BitXor for i16 { fn bitxor(self, other: i16) -> i16 { self ^ other } }
impl BitXor for i32 { fn bitxor(self, other: i32) -> i32 { self ^ other } }
impl BitXor for i64 { fn bitxor(self, other: i64) -> i64 { self ^ other } }

// docs:start:shl-trait
trait Shl {
    fn shl(self, other: Self) -> Self;
}
// docs:end:shl-trait

impl Shl for u8 { fn shl(self, other: u8) -> u8 { self << other } }
impl Shl for u16 { fn shl(self, other: u16) -> u16 { self << other } }
impl Shl for u32 { fn shl(self, other: u32) -> u32 { self << other } }
impl Shl for u64 { fn shl(self, other: u64) -> u64 { self << other } }

// Bit shifting is not currently supported for signed integer types
// impl Shl for i8 { fn shl(self, other: i8) -> i8 { self << other } }
// impl Shl for i16 { fn shl(self, other: i16) -> i16 { self << other } }
// impl Shl for i32 { fn shl(self, other: i32) -> i32 { self << other } }
// impl Shl for i64 { fn shl(self, other: i64) -> i64 { self << other } }

// docs:start:shr-trait
trait Shr {
    fn shr(self, other: Self) -> Self;
}
// docs:end:shr-trait

impl Shr for u8 { fn shr(self, other: u8) -> u8 { self >> other } }
impl Shr for u16 { fn shr(self, other: u16) -> u16 { self >> other } }
impl Shr for u32 { fn shr(self, other: u32) -> u32 { self >> other } }
impl Shr for u64 { fn shr(self, other: u64) -> u64 { self >> other } }

// Bit shifting is not currently supported for signed integer types
// impl Shr for i8 { fn shr(self, other: i8) -> i8 { self >> other } }
// impl Shr for i16 { fn shr(self, other: i16) -> i16 { self >> other } }
// impl Shr for i32 { fn shr(self, other: i32) -> i32 { self >> other } }
// impl Shr for i64 { fn shr(self, other: i64) -> i64 { self >> other } }
