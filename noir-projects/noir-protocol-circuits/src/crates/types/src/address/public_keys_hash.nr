use crate::{
    constants::GENERATOR_INDEX__PARTIAL_ADDRESS, hash::pedersen_hash, grumpkin_point::GrumpkinPoint,
    traits::{ToField, Serialize, Deserialize}
};

// Public keys hash. Used in the computation of an address.
struct PublicKeysHash {
    inner: Field
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

    pub fn compute(public_key: GrumpkinPoint) -> Self {
        PublicKeysHash::from_field(
            pedersen_hash(
                [
            public_key.x,
            public_key.y
        ],
                GENERATOR_INDEX__PARTIAL_ADDRESS
            )
        )
    }

    pub fn to_field(self) -> Field {
        self.inner
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}

#[test]
fn compute_public_keys_hash() {
    let point = GrumpkinPoint { x: 1, y: 2 };
    let actual = PublicKeysHash::compute(point);
    assert(actual.to_field() == 0x1923a6246e305720b6aaf751fde0342613e93c82e455c3831e28375c16dd40d8);
}
