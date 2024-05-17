// docs:start:add-trait
trait Add {
    fn add(self, other: Self) -> Self;
}
// docs:end:add-trait

impl Add for Field { fn add(self, other: Field) -> Field { self + other } }

impl Add for u64 { fn add(self, other: u64) -> u64 { self + other } }
impl Add for u32 { fn add(self, other: u32) -> u32 { self + other } }
impl Add for u16 { fn add(self, other: u16) -> u16 { self + other } }
impl Add for u8 { fn add(self, other: u8) -> u8 { self + other } }

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

impl Sub for u64 { fn sub(self, other: u64) -> u64 { self - other } }
impl Sub for u32 { fn sub(self, other: u32) -> u32 { self - other } }
impl Sub for u16 { fn sub(self, other: u16) -> u16 { self - other } }
impl Sub for u8 { fn sub(self, other: u8) -> u8 { self - other } }

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

impl Mul for u64 { fn mul(self, other: u64) -> u64 { self * other } }
impl Mul for u32 { fn mul(self, other: u32) -> u32 { self * other } }
impl Mul for u16 { fn mul(self, other: u16) -> u16 { self * other } }
impl Mul for u8 { fn mul(self, other: u8) -> u8 { self * other } }

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

impl Div for u64 { fn div(self, other: u64) -> u64 { self / other } }
impl Div for u32 { fn div(self, other: u32) -> u32 { self / other } }
impl Div for u16 { fn div(self, other: u16) -> u16 { self / other } }
impl Div for u8 { fn div(self, other: u8) -> u8 { self / other } }

impl Div for i8 { fn div(self, other: i8) -> i8 { self / other } }
impl Div for i16 { fn div(self, other: i16) -> i16 { self / other } }
impl Div for i32 { fn div(self, other: i32) -> i32 { self / other } }
impl Div for i64 { fn div(self, other: i64) -> i64 { self / other } }

// docs:start:rem-trait
trait Rem{
    fn rem(self, other: Self) -> Self;
}
// docs:end:rem-trait

impl Rem for u64 { fn rem(self, other: u64) -> u64 { self % other } }
impl Rem for u32 { fn rem(self, other: u32) -> u32 { self % other } }
impl Rem for u16 { fn rem(self, other: u16) -> u16 { self % other } }
impl Rem for u8 { fn rem(self, other: u8) -> u8 { self % other } }

impl Rem for i8 { fn rem(self, other: i8) -> i8 { self % other } }
impl Rem for i16 { fn rem(self, other: i16) -> i16 { self % other } }
impl Rem for i32 { fn rem(self, other: i32) -> i32 { self % other } }
impl Rem for i64 { fn rem(self, other: i64) -> i64 { self % other } }

// docs:start:neg-trait
trait Neg {
    fn neg(self) -> Self;
}
// docs:end:neg-trait

// docs:start:neg-trait-impls
impl Neg for Field { fn neg(self) -> Field { -self } }

impl Neg for i8 { fn neg(self) -> i8 { -self } }
impl Neg for i16 { fn neg(self) -> i16 { -self } }
impl Neg for i32 { fn neg(self) -> i32 { -self } }
impl Neg for i64 { fn neg(self) -> i64 { -self } }
// docs:end:neg-trait-impls

