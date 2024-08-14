use dep::protocol_types::{
    address::PublicKeysHash, constants::GENERATOR_INDEX__PUBLIC_KEYS_HASH,
    hash::poseidon2_hash_with_separator, point::Point, traits::{Deserialize, Serialize, Empty, is_empty}
};
use crate::keys::constants::{NUM_KEY_TYPES, NULLIFIER_INDEX, INCOMING_INDEX, OUTGOING_INDEX};

global PUBLIC_KEYS_LENGTH: u32 = 12;

struct PublicKeys {
    npk_m: Point,
    ivpk_m: Point,
    ovpk_m: Point,
    tpk_m: Point,
}

impl Empty for PublicKeys {
    fn empty() -> Self {
        PublicKeys {
            npk_m : Point::empty(),
            ivpk_m : Point::empty(),
            ovpk_m : Point::empty(),
            tpk_m : Point::empty()
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
            poseidon2_hash_with_separator(
                [
                self.npk_m.x,
                self.npk_m.y,
                self.npk_m.is_infinite as Field,
                self.ivpk_m.x,
                self.ivpk_m.y,
                self.ivpk_m.is_infinite as Field,
                self.ovpk_m.x,
                self.ovpk_m.y,
                self.ovpk_m.is_infinite as Field,
                self.tpk_m.x,
                self.tpk_m.y,
                self.tpk_m.is_infinite as Field
            ],
                GENERATOR_INDEX__PUBLIC_KEYS_HASH as Field
            )
        }
        )
    }

    pub fn get_key_by_index(self, index: Field) -> Point {
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
            self.npk_m.is_infinite as Field,
            self.ivpk_m.x,
            self.ivpk_m.y,
            self.ivpk_m.is_infinite as Field,
            self.ovpk_m.x,
            self.ovpk_m.y,
            self.ovpk_m.is_infinite as Field,
            self.tpk_m.x,
            self.tpk_m.y,
            self.tpk_m.is_infinite as Field
        ]
    }
}

impl Deserialize<PUBLIC_KEYS_LENGTH> for PublicKeys {
    fn deserialize(serialized: [Field; PUBLIC_KEYS_LENGTH]) -> PublicKeys {
        PublicKeys {
            npk_m: Point { x:serialized[0], y:serialized[1], is_infinite: serialized[2] as bool },
            ivpk_m: Point { x:serialized[3], y: serialized[4], is_infinite: serialized[5] as bool },
            ovpk_m: Point { x:serialized[6], y: serialized[7], is_infinite: serialized[8] as bool },
            tpk_m: Point { x:serialized[9], y: serialized[10], is_infinite: serialized[11] as bool }
        }
    }
}

#[test]
fn compute_public_keys_hash() {
    let keys = PublicKeys {
        npk_m: Point { x: 1, y: 2, is_infinite: false },
        ivpk_m: Point { x: 3, y: 4, is_infinite: false },
        ovpk_m: Point { x: 5, y: 6, is_infinite: false },
        tpk_m: Point { x: 7, y: 8, is_infinite: false }
    };

    let actual = keys.hash();
    let expected_public_keys_hash = 0x0fecd9a32db731fec1fded1b9ff957a1625c069245a3613a2538bd527068b0ad;

    assert(actual.to_field() == expected_public_keys_hash);
}

#[test]
fn compute_empty_hash() {
    let keys = PublicKeys::empty();

    let actual = keys.hash();
    let test_data_empty_hash = 0x0000000000000000000000000000000000000000000000000000000000000000;

    assert(actual.to_field() == test_data_empty_hash);
}

#[test]
fn test_public_keys_serialization() {
    let keys = PublicKeys {
        npk_m: Point { x: 1, y: 2, is_infinite: false },
        ivpk_m: Point { x: 3, y: 4, is_infinite: false },
        ovpk_m: Point { x: 5, y: 6, is_infinite: false },
        tpk_m: Point { x: 7, y: 8, is_infinite: false }
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
