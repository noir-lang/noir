use crate::utils::field::field_from_bytes;

// Trait: is_empty
//
// The general is_empty trait checks if a data type is is empty,
// and it defines empty for the basic data types as 0.
//
// If a Field is equal to zero, then it is regarded as zero.
// We will go with this definition for now, however it can be problematic 
// if a value can actually be zero. In a future refactor, we can 
// use the optional type for safety. Doing it now would lead to a worse devex
// and would make it harder to sync up with the cpp code.
// Preferred over Default trait to convey intent, as default doesn't necessarily mean empty.
trait Empty {
    fn empty() -> Self;
}

impl Empty for Field { fn empty() -> Self {0} }

impl Empty for u1 { fn empty() -> Self {0} }
impl Empty for u8 { fn empty() -> Self {0} }
impl Empty for u32 { fn empty() -> Self {0} }
impl Empty for u64 { fn empty() -> Self {0} }
impl Empty for U128 { fn empty() -> Self {U128::from_integer(0)} }

pub fn is_empty<T>(item: T) -> bool where T: Empty + Eq {
    item.eq(T::empty())
}

pub fn is_empty_array<T, let N: u32>(array: [T; N]) -> bool where T: Empty + Eq {
    array.all(|elem| is_empty(elem))
}

trait Hash {
    fn hash(self) -> Field;
}

trait ToField {
    fn to_field(self) -> Field;
}

impl ToField for Field {
    fn to_field(self) -> Field {
        self
    }
}

impl ToField for bool { fn to_field(self) -> Field { self as Field } }
impl ToField for u1 { fn to_field(self) -> Field { self as Field } }
impl ToField for u8 { fn to_field(self) -> Field { self as Field } }
impl ToField for u32 { fn to_field(self) -> Field { self as Field } }
impl ToField for u64 { fn to_field(self) -> Field { self as Field } }
impl ToField for U128 {
    fn to_field(self) -> Field {
        self.to_integer()
    }
}
impl<let N: u32> ToField for str<N> {
    fn to_field(self) -> Field {
        assert(N < 32, "String doesn't fit in a field, consider using Serialize instead");
        field_from_bytes(self.as_bytes(), true)
    }
}

trait FromField {
    fn from_field(value: Field) -> Self;
}

impl FromField for Field {
    fn from_field(value: Field) -> Self {
        value
    }
}

impl FromField for bool { fn from_field(value: Field) -> Self { value as bool } }
impl FromField for u1 { fn from_field(value: Field) -> Self { value as u1 } }
impl FromField for u8 { fn from_field(value: Field) -> Self { value as u8 } }
impl FromField for u32 { fn from_field(value: Field) -> Self { value as u32 } }
impl FromField for u64 { fn from_field(value: Field) -> Self { value as u64 } }
impl FromField for U128 {
    fn from_field(value: Field) -> Self {
        U128::from_integer(value)
    }
}

// docs:start:serialize
trait Serialize<let N: u32> {
    fn serialize(self) -> [Field; N];
}
// docs:end:serialize

impl<let N: u32> Serialize<N> for [Field; N] {
    fn serialize(self) -> [Field; N] {
        self
    }
}
impl<let N: u32> Serialize<N> for str<N> {
    fn serialize(self) -> [Field; N] {
        let mut result = [0; N];
        let bytes: [u8; N] = self.as_bytes();
        for i in 0..N {
            result[i] = field_from_bytes([bytes[i];1], true);
        }
        result
    }
}

// docs:start:deserialize
trait Deserialize<let N: u32> {
    fn deserialize(fields: [Field; N]) -> Self;
}
// docs:end:deserialize

impl<let N: u32> Deserialize<N> for [Field; N] {
    fn deserialize(fields: [Field; N]) -> Self {
        fields
    }
}
