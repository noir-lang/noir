use crate::traits::Packable;

global BOOL_PACKED_LEN: u32 = 1;
global U8_PACKED_LEN: u32 = 1;
global U16_PACKED_LEN: u32 = 1;
global U32_PACKED_LEN: u32 = 1;
global U64_PACKED_LEN: u32 = 1;
global U128_PACKED_LEN: u32 = 1;
global FIELD_PACKED_LEN: u32 = 1;
global I8_PACKED_LEN: u32 = 1;
global I16_PACKED_LEN: u32 = 1;
global I32_PACKED_LEN: u32 = 1;
global I64_PACKED_LEN: u32 = 1;

impl Packable<BOOL_PACKED_LEN> for bool {
    fn pack(self) -> [Field; BOOL_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; BOOL_PACKED_LEN]) -> bool {
        fields[0] as bool
    }
}

impl Packable<U8_PACKED_LEN> for u8 {
    fn pack(self) -> [Field; U8_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; U8_PACKED_LEN]) -> Self {
        fields[0] as u8
    }
}

impl Packable<U16_PACKED_LEN> for u16 {
    fn pack(self) -> [Field; U16_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; U16_PACKED_LEN]) -> Self {
        fields[0] as u16
    }
}

impl Packable<U32_PACKED_LEN> for u32 {
    fn pack(self) -> [Field; U32_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; U32_PACKED_LEN]) -> Self {
        fields[0] as u32
    }
}

impl Packable<U64_PACKED_LEN> for u64 {
    fn pack(self) -> [Field; U64_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; U64_PACKED_LEN]) -> Self {
        fields[0] as u64
    }
}

impl Packable<U128_PACKED_LEN> for u128 {
    fn pack(self) -> [Field; U128_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; U128_PACKED_LEN]) -> Self {
        fields[0] as u128
    }
}

impl Packable<FIELD_PACKED_LEN> for Field {
    fn pack(self) -> [Field; FIELD_PACKED_LEN] {
        [self]
    }

    fn unpack(fields: [Field; FIELD_PACKED_LEN]) -> Self {
        fields[0]
    }
}

impl Packable<I8_PACKED_LEN> for i8 {
    fn pack(self) -> [Field; I8_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; I8_PACKED_LEN]) -> Self {
        fields[0] as i8
    }
}

impl Packable<I16_PACKED_LEN> for i16 {
    fn pack(self) -> [Field; I16_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; I16_PACKED_LEN]) -> Self {
        fields[0] as i16
    }
}

impl Packable<I32_PACKED_LEN> for i32 {
    fn pack(self) -> [Field; I32_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; I32_PACKED_LEN]) -> Self {
        fields[0] as i32
    }
}

impl Packable<I64_PACKED_LEN> for i64 {
    fn pack(self) -> [Field; I64_PACKED_LEN] {
        [self as Field]
    }

    fn unpack(fields: [Field; I64_PACKED_LEN]) -> Self {
        fields[0] as i64
    }
}

impl<T, let N: u32, let M: u32> Packable<N * M> for [T; N]
where
    T: Packable<M>,
{
    fn pack(self) -> [Field; N * M] {
        let mut result: [Field; N * M] = std::mem::zeroed();
        let mut serialized: [Field; M] = std::mem::zeroed();
        for i in 0..N {
            serialized = self[i].pack();
            for j in 0..M {
                result[i * M + j] = serialized[j];
            }
        }
        result
    }

    fn unpack(fields: [Field; N * M]) -> Self {
        let mut reader = crate::utils::reader::Reader::new(fields);
        let mut result: [T; N] = std::mem::zeroed();
        reader.read_struct_array::<T, M, N>(Packable::unpack, result)
    }
}

#[test]
fn test_u16_packing() {
    let a: u16 = 10;
    assert_eq(a, u16::unpack(a.pack()));
}

#[test]
fn test_i8_packing() {
    let a: i8 = -10;
    assert_eq(a, i8::unpack(a.pack()));
}

#[test]
fn test_i16_packing() {
    let a: i16 = -10;
    assert_eq(a, i16::unpack(a.pack()));
}

#[test]
fn test_i32_packing() {
    let a: i32 = -10;
    assert_eq(a, i32::unpack(a.pack()));
}

#[test]
fn test_i64_packing() {
    let a: i64 = -10;
    assert_eq(a, i64::unpack(a.pack()));
}
