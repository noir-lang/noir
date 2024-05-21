use dep::protocol_types::{
    address::PublicKeysHash, constants::GENERATOR_INDEX__PUBLIC_KEYS_HASH, hash::poseidon2_hash,
    grumpkin_point::GrumpkinPoint, traits::{Deserialize, Serialize}
};
use crate::keys::constants::{NUM_KEY_TYPES, NULLIFIER_INDEX, INCOMING_INDEX, OUTGOING_INDEX};

global PUBLIC_KEYS_LENGTH = 8;

struct PublicKeys {
    npk_m: GrumpkinPoint,
    ivpk_m: GrumpkinPoint,
    ovpk_m: GrumpkinPoint,
    tpk_m: GrumpkinPoint,
}

impl PublicKeys {
    pub fn hash(self) -> PublicKeysHash {
        PublicKeysHash::from_field(
            poseidon2_hash(
                [
            self.npk_m.x,
            self.npk_m.y,
            self.ivpk_m.x,
            self.ivpk_m.y,
            self.ovpk_m.x,
            self.ovpk_m.y,
            self.tpk_m.x,
            self.tpk_m.y,
            GENERATOR_INDEX__PUBLIC_KEYS_HASH
        ]
            )
        )
    }

    pub fn get_key_by_index(self, index: Field) -> GrumpkinPoint {
        assert(index as u8 < NUM_KEY_TYPES, "Invalid key index");
        if index == NULLIFIER_INDEX {
            self.npk_m
        } else if index == INCOMING_INDEX {
            self.ivpk_m
        } else if index == OUTGOING_INDEX {
            self.ovpk_m
        } else {
            self.tpk_m
        }
    }
}

impl Serialize<PUBLIC_KEYS_LENGTH> for PublicKeys {
    fn serialize(self) -> [Field; PUBLIC_KEYS_LENGTH] {
        [
            self.npk_m.x,
            self.npk_m.y,
            self.ivpk_m.x,
            self.ivpk_m.y,
            self.ovpk_m.x,
            self.ovpk_m.y,
            self.tpk_m.x,
            self.tpk_m.y,
        ]
    }
}

impl Deserialize<PUBLIC_KEYS_LENGTH> for PublicKeys {
    fn deserialize(serialized: [Field; PUBLIC_KEYS_LENGTH]) -> PublicKeys {
        PublicKeys {
            npk_m: GrumpkinPoint { x: serialized[0], y: serialized[1] },
            ivpk_m: GrumpkinPoint { x: serialized[2], y: serialized[3] },
            ovpk_m: GrumpkinPoint { x: serialized[4], y: serialized[5] },
            tpk_m: GrumpkinPoint { x: serialized[6], y: serialized[7] },
        }
    }
}

#[test]
fn compute_public_keys_hash() {
    let keys = PublicKeys {
        npk_m: GrumpkinPoint { x: 1, y: 2 },
        ivpk_m: GrumpkinPoint { x: 3, y: 4 },
        ovpk_m: GrumpkinPoint { x: 5, y: 6 },
        tpk_m: GrumpkinPoint { x: 7, y: 8 }
    };

    let actual = keys.hash();
    let expected_public_keys_hash = 0x1936abe4f6a920d16a9f6917f10a679507687e2cd935dd1f1cdcb1e908c027f3;
    assert(actual.to_field() == expected_public_keys_hash);
}

#[test]
fn test_public_keys_serialization() {
    let keys = PublicKeys {
        npk_m: GrumpkinPoint { x: 1, y: 2 },
        ivpk_m: GrumpkinPoint { x: 3, y: 4 },
        ovpk_m: GrumpkinPoint { x: 5, y: 6 },
        tpk_m: GrumpkinPoint { x: 7, y: 8 }
    };

    let serialized = keys.serialize();
    let deserialized = PublicKeys::deserialize(serialized);

    assert_eq(keys.npk_m.x, deserialized.npk_m.x);
    assert_eq(keys.npk_m.y, deserialized.npk_m.y);
    assert_eq(keys.ivpk_m.x, deserialized.ivpk_m.x);
    assert_eq(keys.ivpk_m.y, deserialized.ivpk_m.y);
    assert_eq(keys.ovpk_m.x, deserialized.ovpk_m.x);
    assert_eq(keys.ovpk_m.y, deserialized.ovpk_m.y);
    assert_eq(keys.tpk_m.x, deserialized.tpk_m.x);
    assert_eq(keys.tpk_m.y, deserialized.tpk_m.y);
}
