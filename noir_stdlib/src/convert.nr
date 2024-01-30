// docs:start:from-trait
trait From<T> {
    fn from(input: T) -> Self;
}
// docs:end:from-trait

impl<T> From<T> for T {
  fn from(input: T) -> T {
      input
  }
}

// docs:start:into-trait
trait Into<T> {
    fn into(input: Self) -> T;
}

impl<T, U> Into<T> for U where T: From<U> {
  fn into(input: U) -> T {
      T::from(input)
  }
}
// docs:end:into-trait

// docs:start:from-impls
// Unsigned integers
impl From<u8> for u16 { fn from(value: u8) -> u16 { value as u16 } }

impl From<u8> for u32 { fn from(value: u8) -> u32 { value as u32 } }
impl From<u16> for u32 { fn from(value: u16) -> u32 { value as u32 } }

impl From<u8> for u64 { fn from(value: u8) -> u64 { value as u64 } }
impl From<u16> for u64 { fn from(value: u16) -> u64 { value as u64 } }
impl From<u32> for u64 { fn from(value: u32) -> u64 { value as u64 } }

impl From<u8> for Field { fn from(value: u8) -> Field { value as Field } }
impl From<u16> for Field { fn from(value: u16) -> Field { value as Field } }
impl From<u32> for Field { fn from(value: u32) -> Field { value as Field } }
impl From<u64> for Field { fn from(value: u64) -> Field { value as Field } }

// Signed integers
impl From<i8> for i16 { fn from(value: i8) -> i16 { value as i16 } }

impl From<i8> for i32 { fn from(value: i8) -> i32 { value as i32 } }
impl From<i16> for i32 { fn from(value: i16) -> i32 { value as i32 } }

impl From<i8> for i64 { fn from(value: i8) -> i64 { value as i64 } }
impl From<i16> for i64 { fn from(value: i16) -> i64 { value as i64 } }
impl From<i32> for i64 { fn from(value: i32) -> i64 { value as i64 } }

// Booleans
impl From<bool> for u8 { fn from(value: bool) -> u8 { value as u8 } }
impl From<bool> for u16 { fn from(value: bool) -> u16 { value as u16 } }
impl From<bool> for u32 { fn from(value: bool) -> u32 { value as u32 } }
impl From<bool> for u64 { fn from(value: bool) -> u64 { value as u64 } }
impl From<bool> for i8 { fn from(value: bool) -> i8 { value as i8 } }
impl From<bool> for i16 { fn from(value: bool) -> i16 { value as i16 } }
impl From<bool> for i32 { fn from(value: bool) -> i32 { value as i32 } }
impl From<bool> for i64 { fn from(value: bool) -> i64 { value as i64 } }
impl From<bool> for Field { fn from(value: bool) -> Field { value as Field } }
// docs:end:from-impls
