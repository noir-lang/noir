// docs:start:not-trait
pub trait Not {
    fn not(self: Self) -> Self;
}
// docs:end:not-trait

// docs:start:not-trait-impls
impl Not for bool {
    fn not(self) -> bool {
        !self
    }
}

impl Not for u128 {
    fn not(self) -> u128 {
        !self
    }
}
impl Not for u64 {
    fn not(self) -> u64 {
        !self
    }
}
impl Not for u32 {
    fn not(self) -> u32 {
        !self
    }
}
impl Not for u16 {
    fn not(self) -> u16 {
        !self
    }
}
impl Not for u8 {
    fn not(self) -> u8 {
        !self
    }
}
impl Not for i8 {
    fn not(self) -> i8 {
        !self
    }
}
impl Not for i16 {
    fn not(self) -> i16 {
        !self
    }
}
impl Not for i32 {
    fn not(self) -> i32 {
        !self
    }
}
impl Not for i64 {
    fn not(self) -> i64 {
        !self
    }
}
// docs:end:not-trait-impls

// docs:start:bitor-trait
pub trait BitOr {
    fn bitor(self, other: Self) -> Self;
}
// docs:end:bitor-trait

impl BitOr for bool {
    fn bitor(self, other: bool) -> bool {
        self | other
    }
}

impl BitOr for u128 {
    fn bitor(self, other: u128) -> u128 {
        self | other
    }
}
impl BitOr for u64 {
    fn bitor(self, other: u64) -> u64 {
        self | other
    }
}
impl BitOr for u32 {
    fn bitor(self, other: u32) -> u32 {
        self | other
    }
}
impl BitOr for u16 {
    fn bitor(self, other: u16) -> u16 {
        self | other
    }
}
impl BitOr for u8 {
    fn bitor(self, other: u8) -> u8 {
        self | other
    }
}
impl BitOr for i8 {
    fn bitor(self, other: i8) -> i8 {
        self | other
    }
}
impl BitOr for i16 {
    fn bitor(self, other: i16) -> i16 {
        self | other
    }
}
impl BitOr for i32 {
    fn bitor(self, other: i32) -> i32 {
        self | other
    }
}
impl BitOr for i64 {
    fn bitor(self, other: i64) -> i64 {
        self | other
    }
}

// docs:start:bitand-trait
pub trait BitAnd {
    fn bitand(self, other: Self) -> Self;
}
// docs:end:bitand-trait

impl BitAnd for bool {
    fn bitand(self, other: bool) -> bool {
        self & other
    }
}

impl BitAnd for u128 {
    fn bitand(self, other: u128) -> u128 {
        self & other
    }
}
impl BitAnd for u64 {
    fn bitand(self, other: u64) -> u64 {
        self & other
    }
}
impl BitAnd for u32 {
    fn bitand(self, other: u32) -> u32 {
        self & other
    }
}
impl BitAnd for u16 {
    fn bitand(self, other: u16) -> u16 {
        self & other
    }
}
impl BitAnd for u8 {
    fn bitand(self, other: u8) -> u8 {
        self & other
    }
}
impl BitAnd for i8 {
    fn bitand(self, other: i8) -> i8 {
        self & other
    }
}
impl BitAnd for i16 {
    fn bitand(self, other: i16) -> i16 {
        self & other
    }
}
impl BitAnd for i32 {
    fn bitand(self, other: i32) -> i32 {
        self & other
    }
}
impl BitAnd for i64 {
    fn bitand(self, other: i64) -> i64 {
        self & other
    }
}

// docs:start:bitxor-trait
pub trait BitXor {
    fn bitxor(self, other: Self) -> Self;
}
// docs:end:bitxor-trait

impl BitXor for bool {
    fn bitxor(self, other: bool) -> bool {
        self ^ other
    }
}

impl BitXor for u128 {
    fn bitxor(self, other: u128) -> u128 {
        self ^ other
    }
}
impl BitXor for u64 {
    fn bitxor(self, other: u64) -> u64 {
        self ^ other
    }
}
impl BitXor for u32 {
    fn bitxor(self, other: u32) -> u32 {
        self ^ other
    }
}
impl BitXor for u16 {
    fn bitxor(self, other: u16) -> u16 {
        self ^ other
    }
}
impl BitXor for u8 {
    fn bitxor(self, other: u8) -> u8 {
        self ^ other
    }
}
impl BitXor for i8 {
    fn bitxor(self, other: i8) -> i8 {
        self ^ other
    }
}
impl BitXor for i16 {
    fn bitxor(self, other: i16) -> i16 {
        self ^ other
    }
}
impl BitXor for i32 {
    fn bitxor(self, other: i32) -> i32 {
        self ^ other
    }
}
impl BitXor for i64 {
    fn bitxor(self, other: i64) -> i64 {
        self ^ other
    }
}

// docs:start:shl-trait
pub trait Shl {
    fn shl(self, other: Self) -> Self;
}
// docs:end:shl-trait

impl Shl for u128 {
    fn shl(self, other: u128) -> u128 {
        self << other
    }
}
impl Shl for u64 {
    fn shl(self, other: u64) -> u64 {
        self << other
    }
}
impl Shl for u32 {
    fn shl(self, other: u32) -> u32 {
        self << other
    }
}
impl Shl for u16 {
    fn shl(self, other: u16) -> u16 {
        self << other
    }
}
impl Shl for u8 {
    fn shl(self, other: u8) -> u8 {
        self << other
    }
}
impl Shl for i8 {
    fn shl(self, other: i8) -> i8 {
        self << other
    }
}
impl Shl for i16 {
    fn shl(self, other: i16) -> i16 {
        self << other
    }
}
impl Shl for i32 {
    fn shl(self, other: i32) -> i32 {
        self << other
    }
}
impl Shl for i64 {
    fn shl(self, other: i64) -> i64 {
        self << other
    }
}

// docs:start:shr-trait
pub trait Shr {
    fn shr(self, other: Self) -> Self;
}
// docs:end:shr-trait

impl Shr for u128 {
    fn shr(self, other: u128) -> u128 {
        self >> other
    }
}
impl Shr for u64 {
    fn shr(self, other: u64) -> u64 {
        self >> other
    }
}
impl Shr for u32 {
    fn shr(self, other: u32) -> u32 {
        self >> other
    }
}
impl Shr for u16 {
    fn shr(self, other: u16) -> u16 {
        self >> other
    }
}
impl Shr for u8 {
    fn shr(self, other: u8) -> u8 {
        self >> other
    }
}
impl Shr for i8 {
    fn shr(self, other: i8) -> i8 {
        self >> other
    }
}
impl Shr for i16 {
    fn shr(self, other: i16) -> i16 {
        self >> other
    }
}
impl Shr for i32 {
    fn shr(self, other: i32) -> i32 {
        self >> other
    }
}
impl Shr for i64 {
    fn shr(self, other: i64) -> i64 {
        self >> other
    }
}

mod tests {
    use crate::ops::bit::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

    #[test]
    fn not() {
        assert_eq(false.not(), true);
        assert_eq(true.not(), false);
        assert_eq(0_u8.not(), 255);
        assert_eq(0_u16.not(), 65535);
        assert_eq(0_u32.not(), 4294967295);
        assert_eq(0_u64.not(), 18446744073709551615);
        assert_eq(0_u128.not(), 340282366920938463463374607431768211455);
        assert_eq(0_i8.not(), -1);
        assert_eq(0_i16.not(), -1);
        assert_eq(0_i32.not(), -1);
        assert_eq(0_i64.not(), -1);
    }

    #[test]
    fn bitor() {
        assert_eq(true.bitor(false), true);
        assert_eq(1_u8.bitor(2), 3);
        assert_eq(1_u16.bitor(2), 3);
        assert_eq(1_u32.bitor(2), 3);
        assert_eq(1_u64.bitor(2), 3);
        assert_eq(1_u128.bitor(2), 3);
        assert_eq(1_i8.bitor(2), 3);
        assert_eq(1_i16.bitor(2), 3);
        assert_eq(1_i32.bitor(2), 3);
        assert_eq(1_i64.bitor(2), 3);
    }

    #[test]
    fn bitand() {
        assert_eq(true.bitand(false), false);
        assert_eq(3_u8.bitand(5), 1);
        assert_eq(3_u16.bitand(5), 1);
        assert_eq(3_u32.bitand(5), 1);
        assert_eq(3_u64.bitand(5), 1);
        assert_eq(3_u128.bitand(5), 1);
        assert_eq(3_i8.bitand(5), 1);
        assert_eq(3_i16.bitand(5), 1);
        assert_eq(3_i32.bitand(5), 1);
        assert_eq(3_i64.bitand(5), 1);
    }

    #[test]
    fn bitxor() {
        assert_eq(true.bitxor(false), true);
        assert_eq(3_u8.bitxor(5), 6);
        assert_eq(3_u16.bitxor(5), 6);
        assert_eq(3_u32.bitxor(5), 6);
        assert_eq(3_u64.bitxor(5), 6);
        assert_eq(3_u128.bitxor(5), 6);
        assert_eq(3_i8.bitxor(5), 6);
        assert_eq(3_i16.bitxor(5), 6);
        assert_eq(3_i32.bitxor(5), 6);
        assert_eq(3_i64.bitxor(5), 6);
    }

    #[test]
    fn shl() {
        assert_eq(1_u8.shl(1), 2);
        assert_eq(1_u16.shl(1), 2);
        assert_eq(1_u32.shl(1), 2);
        assert_eq(1_u64.shl(1), 2);
        assert_eq(1_u128.shl(1), 2);
        assert_eq(1_i8.shl(1), 2);
        assert_eq(1_i16.shl(1), 2);
        assert_eq(1_i32.shl(1), 2);
        assert_eq(1_i64.shl(1), 2);
    }

    #[test]
    fn shr() {
        assert_eq(4_u8.shr(1), 2);
        assert_eq(4_u16.shr(1), 2);
        assert_eq(4_u32.shr(1), 2);
        assert_eq(4_u64.shr(1), 2);
        assert_eq(4_u128.shr(1), 2);
        assert_eq(4_i8.shr(1), 2);
        assert_eq(4_i16.shr(1), 2);
        assert_eq(4_i32.shr(1), 2);
        assert_eq(4_i64.shr(1), 2);
    }
}
