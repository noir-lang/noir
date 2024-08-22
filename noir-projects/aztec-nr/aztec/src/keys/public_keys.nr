use dep::protocol_types::{
    address::PublicKeysHash, constants::GENERATOR_INDEX__PUBLIC_KEYS_HASH,
    hash::poseidon2_hash_with_separator, point::{Point, POINT_LENGTH},
    traits::{Deserialize, Serialize, Empty, is_empty, Hash}
};
use crate::keys::constants::{NULLIFIER_INDEX, INCOMING_INDEX, OUTGOING_INDEX};

global PUBLIC_KEYS_LENGTH: u32 = 12;

struct PublicKeys {
    npk_m: NpkM,
    ivpk_m: IvpkM,
    ovpk_m: OvpkM,
    tpk_m: TpkM,
}

trait ToPoint {
    fn to_point(self) -> Point;
}

struct NpkM {
    inner: Point
}

impl ToPoint for NpkM {
    fn to_point(self) -> Point {
        self.inner
    }
}

impl Serialize<POINT_LENGTH> for NpkM {
    fn serialize(self) -> [Field; POINT_LENGTH] {
        self.inner.serialize()
    }
}

// Note: If we store npk_m_hash directly we can remove this trait implementation. See #8091
impl Hash for NpkM {
    fn hash(self) -> Field {
        self.inner.hash()
    }
}

struct IvpkM {
    inner: Point
}

impl ToPoint for IvpkM {
    fn to_point(self) -> Point {
        self.inner
    }
}

impl Serialize<POINT_LENGTH> for IvpkM {
    fn serialize(self) -> [Field; POINT_LENGTH] {
        self.inner.serialize()
    }
}

struct OvpkM {
    inner: Point
}

impl Hash for OvpkM {
    fn hash(self) -> Field {
        self.inner.hash()
    }
}

impl ToPoint for OvpkM {
    fn to_point(self) -> Point {
        self.inner
    }
}

impl Serialize<POINT_LENGTH> for OvpkM {
    fn serialize(self) -> [Field; POINT_LENGTH] {
        self.inner.serialize()
    }
}

struct TpkM {
    inner: Point
}

impl ToPoint for TpkM {
    fn to_point(self) -> Point {
        self.inner
    }
}

impl Empty for PublicKeys {
    fn empty() -> Self {
        PublicKeys {
            npk_m : NpkM { inner: Point::empty() },
            ivpk_m : IvpkM { inner: Point::empty() },
            ovpk_m : OvpkM { inner: Point::empty() },
            tpk_m : TpkM { inner: Point::empty() }
        }
    }
}

impl Eq for PublicKeys {
    fn eq(self, other: PublicKeys) -> bool {
        ( self.npk_m.inner == other.npk_m.inner ) &
        ( self.ivpk_m.inner == other.ivpk_m.inner ) &
        ( self.ovpk_m.inner == other.ovpk_m.inner ) &
        ( self.tpk_m.inner == other.tpk_m.inner )
    }
}

impl PublicKeys {
    pub fn hash(self) -> PublicKeysHash {
        PublicKeysHash::from_field(
            if is_empty(self) {
            0
        } else {
            poseidon2_hash_with_separator(self.serialize(), GENERATOR_INDEX__PUBLIC_KEYS_HASH as Field)
        }
        )
    }
}

impl Serialize<PUBLIC_KEYS_LENGTH> for PublicKeys {
    fn serialize(self) -> [Field; PUBLIC_KEYS_LENGTH] {
        [
            self.npk_m.inner.x,
            self.npk_m.inner.y,
            self.npk_m.inner.is_infinite as Field,
            self.ivpk_m.inner.x,
            self.ivpk_m.inner.y,
            self.ivpk_m.inner.is_infinite as Field,
            self.ovpk_m.inner.x,
            self.ovpk_m.inner.y,
            self.ovpk_m.inner.is_infinite as Field,
            self.tpk_m.inner.x,
            self.tpk_m.inner.y,
            self.tpk_m.inner.is_infinite as Field
        ]
    }
}

impl Deserialize<PUBLIC_KEYS_LENGTH> for PublicKeys {
    fn deserialize(serialized: [Field; PUBLIC_KEYS_LENGTH]) -> PublicKeys {
        PublicKeys {
            npk_m: NpkM { inner: Point { x:serialized[0], y:serialized[1], is_infinite: serialized[2] as bool } },
            ivpk_m: IvpkM { inner: Point { x:serialized[3], y: serialized[4], is_infinite: serialized[5] as bool } },
            ovpk_m: OvpkM { inner: Point { x:serialized[6], y: serialized[7], is_infinite: serialized[8] as bool } },
            tpk_m: TpkM { inner: Point { x:serialized[9], y: serialized[10], is_infinite: serialized[11] as bool } }
        }
    }
}

#[test]
fn compute_public_keys_hash() {
    let keys = PublicKeys {
        npk_m: NpkM { inner: Point { x: 1, y: 2, is_infinite: false } },
        ivpk_m: IvpkM { inner: Point { x: 3, y: 4, is_infinite: false } },
        ovpk_m: OvpkM { inner: Point { x: 5, y: 6, is_infinite: false } },
        tpk_m: TpkM { inner: Point { x: 7, y: 8, is_infinite: false } }
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
        npk_m: NpkM { inner: Point { x: 1, y: 2, is_infinite: false } },
        ivpk_m: IvpkM { inner: Point { x: 3, y: 4, is_infinite: false } },
        ovpk_m: OvpkM { inner: Point { x: 5, y: 6, is_infinite: false } },
        tpk_m: TpkM { inner: Point { x: 7, y: 8, is_infinite: false } }
    };

    let serialized = keys.serialize();
    let deserialized = PublicKeys::deserialize(serialized);

    assert_eq(keys.npk_m.inner.x, deserialized.npk_m.inner.x);
    assert_eq(keys.npk_m.inner.y, deserialized.npk_m.inner.y);
    assert_eq(keys.ivpk_m.inner.x, deserialized.ivpk_m.inner.x);
    assert_eq(keys.ivpk_m.inner.y, deserialized.ivpk_m.inner.y);
    assert_eq(keys.ovpk_m.inner.x, deserialized.ovpk_m.inner.x);
    assert_eq(keys.ovpk_m.inner.y, deserialized.ovpk_m.inner.y);
    assert_eq(keys.tpk_m.inner.x, deserialized.tpk_m.inner.x);
    assert_eq(keys.tpk_m.inner.y, deserialized.tpk_m.inner.y);
}
