// docs:start:from-trait
pub trait From<T> {
    fn from(input: T) -> Self;
}
// docs:end:from-trait

impl<T> From<T> for T {
    fn from(input: T) -> T {
        input
    }
}

// docs:start:into-trait
pub trait Into<T> {
    fn into(self) -> T;
}

impl<T, U> Into<T> for U
where
    T: From<U>,
{
    fn into(self) -> T {
        T::from(self)
    }
}
// docs:end:into-trait

// docs:start:from-impls
// Unsigned integers

impl From<u8> for u32 {
    fn from(value: u8) -> u32 {
        value as u32
    }
}

impl From<u8> for u64 {
    fn from(value: u8) -> u64 {
        value as u64
    }
}
impl From<u32> for u64 {
    fn from(value: u32) -> u64 {
        value as u64
    }
}

impl From<u8> for Field {
    fn from(value: u8) -> Field {
        value as Field
    }
}
impl From<u32> for Field {
    fn from(value: u32) -> Field {
        value as Field
    }
}
impl From<u64> for Field {
    fn from(value: u64) -> Field {
        value as Field
    }
}

// Signed integers

impl From<i8> for i32 {
    fn from(value: i8) -> i32 {
        value as i32
    }
}

impl From<i8> for i64 {
    fn from(value: i8) -> i64 {
        value as i64
    }
}
impl From<i32> for i64 {
    fn from(value: i32) -> i64 {
        value as i64
    }
}

// Booleans
impl From<bool> for u8 {
    fn from(value: bool) -> u8 {
        value as u8
    }
}
impl From<bool> for u32 {
    fn from(value: bool) -> u32 {
        value as u32
    }
}
impl From<bool> for u64 {
    fn from(value: bool) -> u64 {
        value as u64
    }
}
impl From<bool> for i8 {
    fn from(value: bool) -> i8 {
        value as i8
    }
}
impl From<bool> for i32 {
    fn from(value: bool) -> i32 {
        value as i32
    }
}
impl From<bool> for i64 {
    fn from(value: bool) -> i64 {
        value as i64
    }
}
impl From<bool> for Field {
    fn from(value: bool) -> Field {
        value as Field
    }
}
// docs:end:from-impls

pub(crate) trait CastFromField {
    fn cast_from_field(input: Field) -> Self;
}

impl CastFromField for u8 {
    fn cast_from_field(input: Field) -> u8 {
        input as u8
    }
}

impl CastFromField for u16 {
    fn cast_from_field(input: Field) -> u16 {
        input as u16
    }
}
impl CastFromField for u32 {
    fn cast_from_field(input: Field) -> u32 {
        input as u32
    }
}

impl CastFromField for u64 {
    fn cast_from_field(input: Field) -> u64 {
        input as u64
    }
}

impl CastFromField for i8 {
    fn cast_from_field(input: Field) -> i8 {
        input as i8
    }
}

impl CastFromField for i16 {
    fn cast_from_field(input: Field) -> i16 {
        input as i16
    }
}
impl CastFromField for i32 {
    fn cast_from_field(input: Field) -> i32 {
        input as i32
    }
}

impl CastFromField for i64 {
    fn cast_from_field(input: Field) -> i64 {
        input as i64
    }
}

impl CastFromField for Field {
    fn cast_from_field(input: Field) -> Field {
        input
    }
}
