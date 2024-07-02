use dep::protocol_types::{
    address::PublicKeysHash, constants::GENERATOR_INDEX__PUBLIC_KEYS_HASH, hash::poseidon2_hash,
    grumpkin_point::GrumpkinPoint, traits::{Deserialize, Serialize, Empty, is_empty}
};
use crate::keys::constants::{NUM_KEY_TYPES, NULLIFIER_INDEX, INCOMING_INDEX, OUTGOING_INDEX};

global PUBLIC_KEYS_LENGTH = 8;

struct PublicKeys {
    npk_m: GrumpkinPoint,
    ivpk_m: GrumpkinPoint,
    ovpk_m: GrumpkinPoint,
    tpk_m: GrumpkinPoint,
}

impl Empty for PublicKeys {
    fn empty() -> Self {
        PublicKeys {
            npk_m : GrumpkinPoint::empty(),
            ivpk_m : GrumpkinPoint::empty(),
            ovpk_m : GrumpkinPoint::empty(),
            tpk_m : GrumpkinPoint::empty()
        }
    }
}

impl Eq for PublicKeys {
    fn eq(self, other: PublicKeys) -> bool {
        ( self.npk_m == other.npk_m ) &
        ( self.ivpk_m == other.ivpk_m ) &
        ( self.ovpk_m == other.ovpk_m ) &
        ( self.tpk_m == other.tpk_m )
    }
}

impl PublicKeys {
    pub fn hash(self) -> PublicKeysHash {
        PublicKeysHash::from_field(
            if is_empty(self) {
            0
        } else {
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
        }
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
    let expected_public_keys_hash = 0x2406c1c88b7afc13052335bb9af43fd35034b5ba0a9caab76eda2833cf8ec717;

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
