use crate::traits::{Deserialize, Serialize};

global BOOL_SERIALIZED_LEN: u32 = 1;
global U8_SERIALIZED_LEN: u32 = 1;
global U16_SERIALIZED_LEN: u32 = 1;
global U32_SERIALIZED_LEN: u32 = 1;
global U64_SERIALIZED_LEN: u32 = 1;
global U128_SERIALIZED_LEN: u32 = 1;
global FIELD_SERIALIZED_LEN: u32 = 1;
global I8_SERIALIZED_LEN: u32 = 1;
global I16_SERIALIZED_LEN: u32 = 1;
global I32_SERIALIZED_LEN: u32 = 1;
global I64_SERIALIZED_LEN: u32 = 1;

impl Serialize<BOOL_SERIALIZED_LEN> for bool {
    fn serialize(self) -> [Field; BOOL_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<BOOL_SERIALIZED_LEN> for bool {
    fn deserialize(fields: [Field; BOOL_SERIALIZED_LEN]) -> bool {
        fields[0] as bool
    }
}

impl Serialize<U8_SERIALIZED_LEN> for u8 {
    fn serialize(self) -> [Field; U8_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<U8_SERIALIZED_LEN> for u8 {
    fn deserialize(fields: [Field; U8_SERIALIZED_LEN]) -> Self {
        fields[0] as u8
    }
}

impl Serialize<U16_SERIALIZED_LEN> for u16 {
    fn serialize(self) -> [Field; U16_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<U16_SERIALIZED_LEN> for u16 {
    fn deserialize(fields: [Field; U16_SERIALIZED_LEN]) -> Self {
        fields[0] as u16
    }
}

impl Serialize<U32_SERIALIZED_LEN> for u32 {
    fn serialize(self) -> [Field; U32_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<U32_SERIALIZED_LEN> for u32 {
    fn deserialize(fields: [Field; U32_SERIALIZED_LEN]) -> Self {
        fields[0] as u32
    }
}

impl Serialize<U64_SERIALIZED_LEN> for u64 {
    fn serialize(self) -> [Field; U64_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<U64_SERIALIZED_LEN> for u64 {
    fn deserialize(fields: [Field; U64_SERIALIZED_LEN]) -> Self {
        fields[0] as u64
    }
}

impl Serialize<U128_SERIALIZED_LEN> for u128 {
    fn serialize(self) -> [Field; U128_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<U128_SERIALIZED_LEN> for u128 {
    fn deserialize(fields: [Field; U128_SERIALIZED_LEN]) -> Self {
        fields[0] as u128
    }
}

impl Serialize<FIELD_SERIALIZED_LEN> for Field {
    fn serialize(self) -> [Field; FIELD_SERIALIZED_LEN] {
        [self]
    }
}

impl Deserialize<FIELD_SERIALIZED_LEN> for Field {
    fn deserialize(fields: [Field; FIELD_SERIALIZED_LEN]) -> Self {
        fields[0]
    }
}

impl Serialize<I8_SERIALIZED_LEN> for i8 {
    fn serialize(self) -> [Field; I8_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<I8_SERIALIZED_LEN> for i8 {
    fn deserialize(fields: [Field; I8_SERIALIZED_LEN]) -> Self {
        fields[0] as i8
    }
}

impl Serialize<I16_SERIALIZED_LEN> for i16 {
    fn serialize(self) -> [Field; I16_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<I16_SERIALIZED_LEN> for i16 {
    fn deserialize(fields: [Field; I16_SERIALIZED_LEN]) -> Self {
        fields[0] as i16
    }
}

impl Serialize<I32_SERIALIZED_LEN> for i32 {
    fn serialize(self) -> [Field; I32_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<I32_SERIALIZED_LEN> for i32 {
    fn deserialize(fields: [Field; I32_SERIALIZED_LEN]) -> Self {
        fields[0] as i32
    }
}

impl Serialize<I64_SERIALIZED_LEN> for i64 {
    fn serialize(self) -> [Field; I64_SERIALIZED_LEN] {
        [self as Field]
    }
}

impl Deserialize<I64_SERIALIZED_LEN> for i64 {
    fn deserialize(fields: [Field; I64_SERIALIZED_LEN]) -> Self {
        fields[0] as i64
    }
}

impl<T, let N: u32, let M: u32> Serialize<N * M> for [T; N]
where
    T: Serialize<M>,
{
    fn serialize(self) -> [Field; N * M] {
        let mut result: [Field; N * M] = std::mem::zeroed();
        let mut serialized: [Field; M] = std::mem::zeroed();
        for i in 0..N {
            serialized = self[i].serialize();
            for j in 0..M {
                result[i * M + j] = serialized[j];
            }
        }
        result
    }
}

impl<T, let N: u32, let M: u32> Deserialize<N * M> for [T; N]
where
    T: Deserialize<M>,
{
    fn deserialize(fields: [Field; N * M]) -> Self {
        let mut reader = crate::utils::reader::Reader::new(fields);
        let mut result: [T; N] = std::mem::zeroed();
        reader.read_struct_array::<T, M, N>(Deserialize::deserialize, result)
    }
}

#[test]
fn test_u16_serialization() {
    let a: u16 = 10;
    assert_eq(a, u16::deserialize(a.serialize()));
}

#[test]
fn test_i8_serialization() {
    let a: i8 = -10;
    assert_eq(a, i8::deserialize(a.serialize()));
}

#[test]
fn test_i16_serialization() {
    let a: i16 = -10;
    assert_eq(a, i16::deserialize(a.serialize()));
}

#[test]
fn test_i32_serialization() {
    let a: i32 = -10;
    assert_eq(a, i32::deserialize(a.serialize()));
}

#[test]
fn test_i64_serialization() {
    let a: i64 = -10;
    assert_eq(a, i64::deserialize(a.serialize()));
}
