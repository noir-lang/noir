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

    // TODO(#5830): When we do this refactor, rename compute_new -> compute
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

    // TODO(#5830): When we do this refactor, rename compute_new -> compute
    pub fn compute_new(
        nullifier_public_key: GrumpkinPoint,
        incoming_public_key: GrumpkinPoint,
        outgoing_public_key: GrumpkinPoint,
        tagging_public_key: GrumpkinPoint
    ) -> Self {
        PublicKeysHash::from_field(
            poseidon2_hash(
                [
            nullifier_public_key.x,
            nullifier_public_key.y,
            incoming_public_key.x,
            incoming_public_key.y,
            outgoing_public_key.x,
            outgoing_public_key.y,
            tagging_public_key.x,
            tagging_public_key.y,
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

// TODO(#5830): re-enable this test once the compute function is updated
// #[test]
// fn compute_public_keys_hash() {
//     let point = GrumpkinPoint { x: 1, y: 2 };
//     let actual = PublicKeysHash::compute(point);
//     let expected_public_keys_hash = 0x22d83a089d7650514c2de24cd30185a414d943eaa19817c67bffe2c3183006a3;
//     assert(actual.to_field() == expected_public_keys_hash);
// }
