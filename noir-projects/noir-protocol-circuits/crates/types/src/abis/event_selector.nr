use crate::utils::field::field_from_bytes;
use crate::traits::{Serialize, Deserialize, FromField, ToField, Empty};

global SELECTOR_SIZE = 4;

struct EventSelector {
    // 1st 4-bytes (big-endian leftmost) of abi-encoding of an event.
    inner: u32,
}

impl Eq for EventSelector {
    fn eq(self, other: EventSelector) -> bool {
        other.inner == self.inner
    }
}

impl Serialize<1> for EventSelector {
    fn serialize(self: Self) -> [Field; 1] {
        [self.inner as Field]
    }
}

impl Deserialize<1> for EventSelector {
    fn deserialize(fields: [Field; 1]) -> Self {
        Self {
            inner: fields[0] as u32
        }
    }
}

impl FromField for EventSelector {
    fn from_field(field: Field) -> Self {
        Self { inner: field as u32 }
    }
}

impl ToField for EventSelector {
    fn to_field(self) -> Field {
        self.inner as Field
    }
}

impl Empty for EventSelector {
    fn empty() -> Self {
        Self { inner: 0 as u32 }
    }
}

impl EventSelector {
    pub fn from_u32(value: u32) -> Self {
        Self { inner: value }
    }

    pub fn from_signature<let N: u32>(signature: str<N>) -> Self {
        let bytes = signature.as_bytes();
        let hash = std::hash::keccak256(bytes, bytes.len() as u32);

        let mut selector_be_bytes = [0; SELECTOR_SIZE];
        for i in 0..SELECTOR_SIZE {
            selector_be_bytes[i] = hash[i];
        }

        EventSelector::from_field(field_from_bytes(selector_be_bytes, true))
    }

    pub fn zero() -> Self {
        Self { inner: 0 }
    }
}
