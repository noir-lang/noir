
trait Add {
    fn add(self, other: Self) -> Self;
}

impl Add for Field { fn add(self, other: Field) -> Field { self + other } }

impl Add for u8 { fn add(self, other: u8) -> u8 { self + other } }
impl Add for u16 { fn add(self, other: u16) -> u16 { self + other } }
impl Add for u32 { fn add(self, other: u32) -> u32 { self + other } }
impl Add for u64 { fn add(self, other: u64) -> u64 { self + other } }

impl Add for i8 { fn add(self, other: i8) -> i8 { self + other } }
impl Add for i16 { fn add(self, other: i16) -> i16 { self + other } }
impl Add for i32 { fn add(self, other: i32) -> i32 { self + other } }
impl Add for i64 { fn add(self, other: i64) -> i64 { self + other } }

trait Sub {
    fn sub(self, other: Self) -> Self;
}

impl Sub for Field { fn sub(self, other: Field) -> Field { self - other } }

impl Sub for u8 { fn sub(self, other: u8) -> u8 { self - other } }
impl Sub for u16 { fn sub(self, other: u16) -> u16 { self - other } }
impl Sub for u32 { fn sub(self, other: u32) -> u32 { self - other } }
impl Sub for u64 { fn sub(self, other: u64) -> u64 { self - other } }

impl Sub for i8 { fn sub(self, other: i8) -> i8 { self - other } }
impl Sub for i16 { fn sub(self, other: i16) -> i16 { self - other } }
impl Sub for i32 { fn sub(self, other: i32) -> i32 { self - other } }
impl Sub for i64 { fn sub(self, other: i64) -> i64 { self - other } }

trait Mul {
    fn mul(self, other: Self) -> Self;
}

impl Mul for Field { fn mul(self, other: Field) -> Field { self * other } }

impl Mul for u8 { fn mul(self, other: u8) -> u8 { self * other } }
impl Mul for u16 { fn mul(self, other: u16) -> u16 { self * other } }
impl Mul for u32 { fn mul(self, other: u32) -> u32 { self * other } }
impl Mul for u64 { fn mul(self, other: u64) -> u64 { self * other } }

impl Mul for i8 { fn mul(self, other: i8) -> i8 { self * other } }
impl Mul for i16 { fn mul(self, other: i16) -> i16 { self * other } }
impl Mul for i32 { fn mul(self, other: i32) -> i32 { self * other } }
impl Mul for i64 { fn mul(self, other: i64) -> i64 { self * other } }

trait Div {
    fn div(self, other: Self) -> Self;
}

impl Div for Field { fn div(self, other: Field) -> Field { self / other } }

impl Div for u8 { fn div(self, other: u8) -> u8 { self / other } }
impl Div for u16 { fn div(self, other: u16) -> u16 { self / other } }
impl Div for u32 { fn div(self, other: u32) -> u32 { self / other } }
impl Div for u64 { fn div(self, other: u64) -> u64 { self / other } }

impl Div for i8 { fn div(self, other: i8) -> i8 { self / other } }
impl Div for i16 { fn div(self, other: i16) -> i16 { self / other } }
impl Div for i32 { fn div(self, other: i32) -> i32 { self / other } }
impl Div for i64 { fn div(self, other: i64) -> i64 { self / other } }

trait Eq {
    fn eq(self, other: Self) -> bool;
}

impl Eq for Field { fn eq(self, other: Field) -> bool { self == other } }

impl Eq for u8 { fn eq(self, other: u8) -> bool { self == other } }
impl Eq for u16 { fn eq(self, other: u16) -> bool { self == other } }
impl Eq for u32 { fn eq(self, other: u32) -> bool { self == other } }
impl Eq for u64 { fn eq(self, other: u64) -> bool { self == other } }

impl Eq for i8 { fn eq(self, other: i8) -> bool { self == other } }
impl Eq for i16 { fn eq(self, other: i16) -> bool { self == other } }
impl Eq for i32 { fn eq(self, other: i32) -> bool { self == other } }
impl Eq for i64 { fn eq(self, other: i64) -> bool { self == other } }

impl Eq for () { fn eq(_self: Self, _other: ()) -> bool { true } }
impl Eq for bool { fn eq(self, other: bool) -> bool { self == other } }

impl<T, N> Eq for [T; N] where T: Eq {
    fn eq(self, other: [T; N]) -> bool {
        let mut result = true;
        for i in 0 .. self.len() {
            result &= self[i].eq(other[i]);
        }
        result
    }
}

impl<A, B> Eq for (A, B) where A: Eq, B: Eq {
    fn eq(self, other: (A, B)) -> bool {
        self.0.eq(other.0) & self.1.eq(other.1)
    }
}

impl<A, B, C> Eq for (A, B, C) where A: Eq, B: Eq, C: Eq {
    fn eq(self, other: (A, B, C)) -> bool {
        self.0.eq(other.0) & self.1.eq(other.1) & self.2.eq(other.2)
    }
}

impl<A, B, C, D> Eq for (A, B, C, D) where A: Eq, B: Eq, C: Eq, D: Eq {
    fn eq(self, other: (A, B, C, D)) -> bool {
        self.0.eq(other.0) & self.1.eq(other.1) & self.2.eq(other.2) & self.3.eq(other.3)
    }
}

impl<A, B, C, D, E> Eq for (A, B, C, D, E) where A: Eq, B: Eq, C: Eq, D: Eq, E: Eq {
    fn eq(self, other: (A, B, C, D, E)) -> bool {
        self.0.eq(other.0) & self.1.eq(other.1) & self.2.eq(other.2) & self.3.eq(other.3) & self.4.eq(other.4)
    }
}
