use crate::{
    address::{
        partial_address::PartialAddress, salted_initialization_hash::SaltedInitializationHash,
    },
    constants::{
        AZTEC_ADDRESS_LENGTH, GENERATOR_INDEX__CONTRACT_ADDRESS_V1, MAX_FIELD_VALUE,
        MAX_PROTOCOL_CONTRACTS,
    },
    contract_class_id::ContractClassId,
    hash::poseidon2_hash_with_separator,
    public_keys::{IvpkM, NpkM, OvpkM, PublicKeys, ToPoint, TpkM},
    traits::{Deserialize, Empty, FromField, Packable, Serialize, ToField},
    utils::field::{pow, sqrt},
};

// We do below because `use crate::point::Point;` does not work
use dep::std::embedded_curve_ops::EmbeddedCurvePoint as Point;

use crate::public_keys::AddressPoint;
use std::{
    embedded_curve_ops::{EmbeddedCurveScalar, fixed_base_scalar_mul as derive_public_key},
    ops::Add,
};

// Aztec address
pub struct AztecAddress {
    pub inner: Field,
}

impl Eq for AztecAddress {
    fn eq(self, other: Self) -> bool {
        self.to_field() == other.to_field()
    }
}

impl Empty for AztecAddress {
    fn empty() -> Self {
        Self { inner: 0 }
    }
}

impl ToField for AztecAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl FromField for AztecAddress {
    fn from_field(value: Field) -> AztecAddress {
        AztecAddress { inner: value }
    }
}

impl Serialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn serialize(self: Self) -> [Field; AZTEC_ADDRESS_LENGTH] {
        [self.to_field()]
    }
}

impl Deserialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn deserialize(fields: [Field; AZTEC_ADDRESS_LENGTH]) -> Self {
        FromField::from_field(fields[0])
    }
}

/// We implement the Packable trait for AztecAddress because it can be stored in contract's storage (and there
/// the implementation of Packable is required).
impl Packable<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn pack(self) -> [Field; AZTEC_ADDRESS_LENGTH] {
        self.serialize()
    }

    fn unpack(fields: [Field; AZTEC_ADDRESS_LENGTH]) -> Self {
        Self::deserialize(fields)
    }
}

impl AztecAddress {
    pub fn zero() -> Self {
        Self { inner: 0 }
    }

    pub fn to_address_point(self) -> AddressPoint {
        // We compute the address point by taking our address, setting it to x, and then solving for y in the
        // equation which defines our bn curve:
        // y^2 = x^3 - 17; x = address
        let x = self.inner;
        let y_squared = pow(x, 3) - 17;

        // TODO (#8970): Handle cases where we cannot recover a point from an address
        let mut y = sqrt(y_squared);

        // If we get a negative y coordinate (any y where y > MAX_FIELD_VALUE / 2), we pin it to the
        // positive one (any value where y <= MAX_FIELD_VALUE / 2) by subtracting it from the Field modulus
        // note: The field modulus is MAX_FIELD_VALUE + 1
        if (!(y.lt(MAX_FIELD_VALUE / 2) | y.eq(MAX_FIELD_VALUE / 2))) {
            y = (MAX_FIELD_VALUE + 1) - y;
        }

        AddressPoint { inner: Point { x: self.inner, y, is_infinite: false } }
    }

    pub fn compute(public_keys: PublicKeys, partial_address: PartialAddress) -> AztecAddress {
        let public_keys_hash = public_keys.hash();

        let pre_address = poseidon2_hash_with_separator(
            [public_keys_hash.to_field(), partial_address.to_field()],
            GENERATOR_INDEX__CONTRACT_ADDRESS_V1,
        );

        let address_point = derive_public_key(EmbeddedCurveScalar::from_field(pre_address)).add(
            public_keys.ivpk_m.to_point(),
        );

        // Note that our address is only the x-coordinate of the full address_point. This is okay because when people want to encrypt something and send it to us
        // they can recover our full point using the x-coordinate (our address itself). To do this, they recompute the y-coordinate according to the equation y^2 = x^3 - 17.
        // When they do this, they may get a positive y-coordinate (a value that is less than or equal to MAX_FIELD_VALUE / 2) or
        // a negative y-coordinate (a value that is more than MAX_FIELD_VALUE), and we cannot dictate which one they get and hence the recovered point may sometimes be different than the one
        // our secret can decrypt. Regardless though, they should and will always encrypt using point with the positive y-coordinate by convention.
        // This ensures that everyone encrypts to the same point given an arbitrary x-coordinate (address). This is allowed because even though our original point may not have a positive y-coordinate,
        // with our original secret, we will be able to derive the secret to the point with the flipped (and now positive) y-coordinate that everyone encrypts to.
        AztecAddress::from_field(address_point.x)
    }

    pub fn compute_from_class_id(
        contract_class_id: ContractClassId,
        salted_initialization_hash: SaltedInitializationHash,
        public_keys: PublicKeys,
    ) -> Self {
        let partial_address = PartialAddress::compute_from_salted_initialization_hash(
            contract_class_id,
            salted_initialization_hash,
        );

        AztecAddress::compute(public_keys, partial_address)
    }

    pub fn is_protocol_contract(self) -> bool {
        self.inner.lt(MAX_PROTOCOL_CONTRACTS as Field)
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}

#[test]
fn compute_address_from_partial_and_pub_keys() {
    let public_keys = PublicKeys {
        npk_m: NpkM {
            inner: Point {
                x: 0x22f7fcddfa3ce3e8f0cc8e82d7b94cdd740afa3e77f8e4a63ea78a239432dcab,
                y: 0x0471657de2b6216ade6c506d28fbc22ba8b8ed95c871ad9f3e3984e90d9723a7,
                is_infinite: false,
            },
        },
        ivpk_m: IvpkM {
            inner: Point {
                x: 0x111223493147f6785514b1c195bb37a2589f22a6596d30bb2bb145fdc9ca8f1e,
                y: 0x273bbffd678edce8fe30e0deafc4f66d58357c06fd4a820285294b9746c3be95,
                is_infinite: false,
            },
        },
        ovpk_m: OvpkM {
            inner: Point {
                x: 0x09115c96e962322ffed6522f57194627136b8d03ac7469109707f5e44190c484,
                y: 0x0c49773308a13d740a7f0d4f0e6163b02c5a408b6f965856b6a491002d073d5b,
                is_infinite: false,
            },
        },
        tpk_m: TpkM {
            inner: Point {
                x: 0x00d3d81beb009873eb7116327cf47c612d5758ef083d4fda78e9b63980b2a762,
                y: 0x2f567d22d2b02fe1f4ad42db9d58a36afd1983e7e2909d1cab61cafedad6193a,
                is_infinite: false,
            },
        },
    };

    let partial_address = PartialAddress::from_field(
        0x0a7c585381b10f4666044266a02405bf6e01fa564c8517d4ad5823493abd31de,
    );

    let address = AztecAddress::compute(public_keys, partial_address);

    // The following value was generated by `derivation.test.ts`.
    // --> Run the test with AZTEC_GENERATE_TEST_DATA=1 flag to update test data.
    let expected_computed_address_from_partial_and_pubkeys =
        0x24e4646f58b9fbe7d38e317db8d5636c423fbbdfbe119fc190fe9c64747e0c62;
    assert(address.to_field() == expected_computed_address_from_partial_and_pubkeys);
}

#[test]
fn compute_preaddress_from_partial_and_pub_keys() {
    let pre_address = poseidon2_hash_with_separator([1, 2], GENERATOR_INDEX__CONTRACT_ADDRESS_V1);
    let expected_computed_preaddress_from_partial_and_pubkey =
        0x23ce9be3fa3c846b0f9245cc796902e731d04f086e8a42473bb29e405fc98075;
    assert(pre_address == expected_computed_preaddress_from_partial_and_pubkey);
}

#[test]
fn from_field_to_field() {
    let address = AztecAddress { inner: 37 };
    assert_eq(FromField::from_field(address.to_field()), address);
}

#[test]
fn serde() {
    let address = AztecAddress { inner: 37 };
    assert_eq(Deserialize::deserialize(address.serialize()), address);
}
