use crate::{
    address::public_keys_hash::PublicKeysHash,
    constants::{
        DEFAULT_IVPK_M_X, DEFAULT_IVPK_M_Y, DEFAULT_NPK_M_X, DEFAULT_NPK_M_Y, DEFAULT_OVPK_M_X,
        DEFAULT_OVPK_M_Y, DEFAULT_TPK_M_X, DEFAULT_TPK_M_Y, GENERATOR_INDEX__PUBLIC_KEYS_HASH,
    },
    hash::poseidon2_hash_with_separator,
    point::POINT_LENGTH,
    traits::{Deserialize, Hash, Serialize},
};

use dep::std::embedded_curve_ops::EmbeddedCurvePoint as Point;
use std::default::Default;

pub global PUBLIC_KEYS_LENGTH: u32 = 12;

pub struct PublicKeys {
    pub npk_m: NpkM,
    pub ivpk_m: IvpkM,
    pub ovpk_m: OvpkM,
    pub tpk_m: TpkM,
}

pub trait ToPoint {
    fn to_point(self) -> Point;
}

pub struct NpkM {
    pub inner: Point,
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

pub struct IvpkM {
    pub inner: Point,
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

pub struct OvpkM {
    pub inner: Point,
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

pub struct TpkM {
    pub inner: Point,
}

impl ToPoint for TpkM {
    fn to_point(self) -> Point {
        self.inner
    }
}

impl Serialize<POINT_LENGTH> for TpkM {
    fn serialize(self) -> [Field; POINT_LENGTH] {
        self.inner.serialize()
    }
}

impl Default for PublicKeys {
    fn default() -> Self {
        PublicKeys {
            npk_m: NpkM {
                inner: Point { x: DEFAULT_NPK_M_X, y: DEFAULT_NPK_M_Y, is_infinite: false },
            },
            ivpk_m: IvpkM {
                inner: Point { x: DEFAULT_IVPK_M_X, y: DEFAULT_IVPK_M_Y, is_infinite: false },
            },
            ovpk_m: OvpkM {
                inner: Point { x: DEFAULT_OVPK_M_X, y: DEFAULT_OVPK_M_Y, is_infinite: false },
            },
            tpk_m: TpkM {
                inner: Point { x: DEFAULT_TPK_M_X, y: DEFAULT_TPK_M_Y, is_infinite: false },
            },
        }
    }
}

impl Eq for PublicKeys {
    fn eq(self, other: PublicKeys) -> bool {
        (self.npk_m.inner == other.npk_m.inner)
            & (self.ivpk_m.inner == other.ivpk_m.inner)
            & (self.ovpk_m.inner == other.ovpk_m.inner)
            & (self.tpk_m.inner == other.tpk_m.inner)
    }
}

impl PublicKeys {
    pub fn hash(self) -> PublicKeysHash {
        PublicKeysHash::from_field(poseidon2_hash_with_separator(
            self.serialize(),
            GENERATOR_INDEX__PUBLIC_KEYS_HASH as Field,
        ))
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
            self.tpk_m.inner.is_infinite as Field,
        ]
    }
}

impl Deserialize<PUBLIC_KEYS_LENGTH> for PublicKeys {
    fn deserialize(serialized: [Field; PUBLIC_KEYS_LENGTH]) -> PublicKeys {
        PublicKeys {
            npk_m: NpkM {
                inner: Point {
                    x: serialized[0],
                    y: serialized[1],
                    is_infinite: serialized[2] as bool,
                },
            },
            ivpk_m: IvpkM {
                inner: Point {
                    x: serialized[3],
                    y: serialized[4],
                    is_infinite: serialized[5] as bool,
                },
            },
            ovpk_m: OvpkM {
                inner: Point {
                    x: serialized[6],
                    y: serialized[7],
                    is_infinite: serialized[8] as bool,
                },
            },
            tpk_m: TpkM {
                inner: Point {
                    x: serialized[9],
                    y: serialized[10],
                    is_infinite: serialized[11] as bool,
                },
            },
        }
    }
}

pub struct AddressPoint {
    pub inner: Point,
}

impl ToPoint for AddressPoint {
    fn to_point(self) -> Point {
        self.inner
    }
}

#[test]
unconstrained fn compute_public_keys_hash() {
    let keys = PublicKeys {
        npk_m: NpkM { inner: Point { x: 1, y: 2, is_infinite: false } },
        ivpk_m: IvpkM { inner: Point { x: 3, y: 4, is_infinite: false } },
        ovpk_m: OvpkM { inner: Point { x: 5, y: 6, is_infinite: false } },
        tpk_m: TpkM { inner: Point { x: 7, y: 8, is_infinite: false } },
    };

    let actual = keys.hash();
    let expected_public_keys_hash =
        0x0fecd9a32db731fec1fded1b9ff957a1625c069245a3613a2538bd527068b0ad;

    assert(actual.to_field() == expected_public_keys_hash);
}

#[test]
unconstrained fn compute_default_hash() {
    let keys = PublicKeys::default();

    let actual = keys.hash();
    let test_data_default_hash = 0x1d3bf1fb93ae0e9cda83b203dd91c3bfb492a9aecf30ec90e1057eced0f0e62d;

    assert(actual.to_field() == test_data_default_hash);
}

#[test]
unconstrained fn test_public_keys_serialization() {
    let keys = PublicKeys {
        npk_m: NpkM { inner: Point { x: 1, y: 2, is_infinite: false } },
        ivpk_m: IvpkM { inner: Point { x: 3, y: 4, is_infinite: false } },
        ovpk_m: OvpkM { inner: Point { x: 5, y: 6, is_infinite: false } },
        tpk_m: TpkM { inner: Point { x: 7, y: 8, is_infinite: false } },
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
