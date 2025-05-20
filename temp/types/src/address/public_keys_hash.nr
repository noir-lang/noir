use crate::traits::{Deserialize, Serialize, ToField};

// Public keys hash. Used in the computation of an address.
pub struct PublicKeysHash {
    inner: Field,
}

impl ToField for PublicKeysHash {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl Serialize<1> for PublicKeysHash {
    fn serialize(self: Self) -> [Field; 1] {
        [self.to_field()]
    }
}

impl Deserialize<1> for PublicKeysHash {
    fn deserialize(fields: [Field; 1]) -> Self {
        PublicKeysHash::from_field(fields[0])
    }
}

impl Eq for PublicKeysHash {
    fn eq(self, other: Self) -> bool {
        self.inner == other.inner
    }
}

impl PublicKeysHash {
    pub fn from_field(field: Field) -> Self {
        Self { inner: field }
    }

    pub fn to_field(self) -> Field {
        self.inner
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}
