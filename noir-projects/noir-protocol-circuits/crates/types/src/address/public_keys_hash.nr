use crate::{
    constants::{GENERATOR_INDEX__PARTIAL_ADDRESS, GENERATOR_INDEX__PUBLIC_KEYS_HASH},
    hash::pedersen_hash, grumpkin_point::GrumpkinPoint, traits::{ToField, Serialize, Deserialize},
    hash::poseidon2_hash
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

    pub fn compute(npk_m: GrumpkinPoint, ivpk_m: GrumpkinPoint, ovpk_m: GrumpkinPoint, tpk_m: GrumpkinPoint) -> Self {
        PublicKeysHash::from_field(
            poseidon2_hash(
                [
            npk_m.x,
            npk_m.y,
            ivpk_m.x,
            ivpk_m.y,
            ovpk_m.x,
            ovpk_m.y,
            tpk_m.x,
            tpk_m.y,
            GENERATOR_INDEX__PUBLIC_KEYS_HASH
        ]
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
    let npk_m = GrumpkinPoint { x: 1, y: 2 };
    let ivpk_m = GrumpkinPoint { x: 3, y: 4 };
    let ovpk_m = GrumpkinPoint { x: 5, y: 6 };
    let tpk_m = GrumpkinPoint { x: 7, y: 8 };

    let actual = PublicKeysHash::compute(npk_m, ivpk_m, ovpk_m, tpk_m);
    let expected_public_keys_hash = 0x1936abe4f6a920d16a9f6917f10a679507687e2cd935dd1f1cdcb1e908c027f3;
    assert(actual.to_field() == expected_public_keys_hash);
}
