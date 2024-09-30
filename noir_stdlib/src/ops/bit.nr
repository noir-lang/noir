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
impl Not for u1 {
    fn not(self) -> u1 {
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
impl BitOr for u1 {
    fn bitor(self, other: u1) -> u1 {
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
impl BitAnd for u1 {
    fn bitand(self, other: u1) -> u1 {
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
impl BitXor for u1 {
    fn bitxor(self, other: u1) -> u1 {
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
    fn shl(self, other: u8) -> Self;
}
// docs:end:shl-trait

impl Shl for u32 {
    fn shl(self, other: u8) -> u32 {
        self << other
    }
}
impl Shl for u64 {
    fn shl(self, other: u8) -> u64 {
        self << other
    }
}
impl Shl for u16 {
    fn shl(self, other: u8) -> u16 {
        self << other
    }
}
impl Shl for u8 {
    fn shl(self, other: u8) -> u8 {
        self << other
    }
}
impl Shl for u1 {
    fn shl(self, other: u8) -> u1 {
        self << other
    }
}

impl Shl for i8 {
    fn shl(self, other: u8) -> i8 {
        self << other
    }
}
impl Shl for i16 {
    fn shl(self, other: u8) -> i16 {
        self << other
    }
}
impl Shl for i32 {
    fn shl(self, other: u8) -> i32 {
        self << other
    }
}
impl Shl for i64 {
    fn shl(self, other: u8) -> i64 {
        self << other
    }
}

// docs:start:shr-trait
pub trait Shr {
    fn shr(self, other: u8) -> Self;
}
// docs:end:shr-trait

impl Shr for u64 {
    fn shr(self, other: u8) -> u64 {
        self >> other
    }
}
impl Shr for u32 {
    fn shr(self, other: u8) -> u32 {
        self >> other
    }
}
impl Shr for u16 {
    fn shr(self, other: u8) -> u16 {
        self >> other
    }
}
impl Shr for u8 {
    fn shr(self, other: u8) -> u8 {
        self >> other
    }
}
impl Shr for u1 {
    fn shr(self, other: u8) -> u1 {
        self >> other
    }
}

impl Shr for i8 {
    fn shr(self, other: u8) -> i8 {
        self >> other
    }
}
impl Shr for i16 {
    fn shr(self, other: u8) -> i16 {
        self >> other
    }
}
impl Shr for i32 {
    fn shr(self, other: u8) -> i32 {
        self >> other
    }
}
impl Shr for i64 {
    fn shr(self, other: u8) -> i64 {
        self >> other
    }
}

