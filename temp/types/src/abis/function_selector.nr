use crate::traits::{Deserialize, Empty, FromField, Serialize, ToField};

pub struct FunctionSelector {
    // 1st 4-bytes of abi-encoding of function.
    pub inner: u32,
}

impl Eq for FunctionSelector {
    fn eq(self, function_selector: FunctionSelector) -> bool {
        function_selector.inner == self.inner
    }
}

impl Serialize<1> for FunctionSelector {
    fn serialize(self: Self) -> [Field; 1] {
        [self.inner as Field]
    }
}

impl Deserialize<1> for FunctionSelector {
    fn deserialize(fields: [Field; 1]) -> Self {
        Self { inner: fields[0] as u32 }
    }
}

impl FromField for FunctionSelector {
    fn from_field(field: Field) -> Self {
        Self { inner: field as u32 }
    }
}

impl ToField for FunctionSelector {
    fn to_field(self) -> Field {
        self.inner as Field
    }
}

impl Empty for FunctionSelector {
    fn empty() -> Self {
        Self { inner: 0 as u32 }
    }
}

impl FunctionSelector {
    pub fn from_u32(value: u32) -> Self {
        Self { inner: value }
    }

    pub fn from_signature<let N: u32>(signature: str<N>) -> Self {
        let bytes = signature.as_bytes();
        let hash = crate::hash::poseidon2_hash_bytes(bytes);

        // `hash` is automatically truncated to fit within 32 bits.
        FunctionSelector::from_field(hash)
    }

    pub fn zero() -> Self {
        Self { inner: 0 }
    }
}

#[test]
fn test_is_valid_selector() {
    let selector = FunctionSelector::from_signature("IS_VALID()");
    assert_eq(selector.to_field(), 0x73cdda47);
}

#[test]
fn test_long_selector() {
    let selector =
        FunctionSelector::from_signature("foo_and_bar_and_baz_and_foo_bar_baz_and_bar_foo");
    assert_eq(selector.to_field(), 0x7590a997);
}
