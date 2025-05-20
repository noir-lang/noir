use crate::{
    abis::{read_request::ScopedReadRequest, side_effect::{Ordered, OrderedValue, Readable, Scoped}},
    address::AztecAddress,
    constants::{NULLIFIER_LENGTH, SCOPED_NULLIFIER_LENGTH},
    hash::compute_siloed_nullifier,
    traits::{Deserialize, Empty, Serialize, ToField},
    utils::{arrays::array_concat, reader::Reader},
};

pub struct Nullifier {
    pub value: Field,
    pub counter: u32,
    pub note_hash: Field,
}

impl Ordered for Nullifier {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl OrderedValue<Field> for Nullifier {
    fn value(self) -> Field {
        self.value
    }
}

impl Eq for Nullifier {
    fn eq(self, other: Nullifier) -> bool {
        (self.value == other.value)
            & (self.counter == other.counter)
            & (self.note_hash == other.note_hash)
    }
}

impl Empty for Nullifier {
    fn empty() -> Self {
        Nullifier { value: 0, counter: 0, note_hash: 0 }
    }
}

impl Serialize<NULLIFIER_LENGTH> for Nullifier {
    fn serialize(self) -> [Field; NULLIFIER_LENGTH] {
        [self.value, self.counter as Field, self.note_hash]
    }
}

impl Deserialize<NULLIFIER_LENGTH> for Nullifier {
    fn deserialize(values: [Field; NULLIFIER_LENGTH]) -> Self {
        Self { value: values[0], counter: values[1] as u32, note_hash: values[2] }
    }
}

impl Readable<ScopedReadRequest> for Nullifier {
    fn assert_match_read_request(self, read_request: ScopedReadRequest) {
        // Public kernels output Nullifier instead of ScopedNullifier.
        // The nullifier value has been siloed.
        let siloed_request_value =
            compute_siloed_nullifier(read_request.contract_address, read_request.value());
        assert_eq(
            self.value,
            siloed_request_value,
            "Value of the nullifier does not match read request",
        );
        assert(
            read_request.counter() > self.counter,
            "Read request counter must be greater than the counter of the nullifier",
        );
    }
}

impl Nullifier {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedNullifier {
        ScopedNullifier { nullifier: self, contract_address }
    }
}

pub struct ScopedNullifier {
    pub nullifier: Nullifier,
    pub contract_address: AztecAddress,
}

impl Scoped<Nullifier> for ScopedNullifier {
    fn inner(self) -> Nullifier {
        self.nullifier
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Ordered for ScopedNullifier {
    fn counter(self) -> u32 {
        self.nullifier.counter
    }
}

impl OrderedValue<Field> for ScopedNullifier {
    fn value(self) -> Field {
        self.nullifier.value
    }
}

impl Eq for ScopedNullifier {
    fn eq(self, other: ScopedNullifier) -> bool {
        (self.nullifier == other.nullifier) & (self.contract_address == other.contract_address)
    }
}

impl Empty for ScopedNullifier {
    fn empty() -> Self {
        ScopedNullifier { nullifier: Nullifier::empty(), contract_address: AztecAddress::empty() }
    }
}

impl Serialize<SCOPED_NULLIFIER_LENGTH> for ScopedNullifier {
    fn serialize(self) -> [Field; SCOPED_NULLIFIER_LENGTH] {
        array_concat(
            self.nullifier.serialize(),
            [self.contract_address.to_field()],
        )
    }
}

impl Deserialize<SCOPED_NULLIFIER_LENGTH> for ScopedNullifier {
    fn deserialize(values: [Field; SCOPED_NULLIFIER_LENGTH]) -> Self {
        let mut reader = Reader::new(values);
        let res = Self {
            nullifier: reader.read_struct(Nullifier::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        res
    }
}

impl Readable<ScopedReadRequest> for ScopedNullifier {
    fn assert_match_read_request(self, read_request: ScopedReadRequest) {
        assert_eq(
            self.nullifier.value,
            read_request.value(),
            "Value of the nullifier does not match read request",
        );
        assert_eq(
            self.contract_address,
            read_request.contract_address,
            "Contract address of the nullifier does not match read request",
        );
        assert(
            read_request.counter() > self.nullifier.counter,
            "Read request counter must be greater than the counter of the nullifier",
        );
    }
}

impl ScopedNullifier {
    pub fn nullified_note_hash(self) -> Field {
        self.nullifier.note_hash
    }

    pub fn expose_to_public(self) -> Nullifier {
        // Hide the actual counter and note hash when exposing it to the public kernel.
        Nullifier { value: self.nullifier.value, counter: 0, note_hash: 0 }
    }
}

#[test]
fn serialization_of_empty() {
    let item = Nullifier::empty();
    let serialized = item.serialize();
    let deserialized = Nullifier::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn serialization_of_empty_scoped() {
    let item = ScopedNullifier::empty();
    let serialized = item.serialize();
    let deserialized = ScopedNullifier::deserialize(serialized);
    assert(item.eq(deserialized));
}
