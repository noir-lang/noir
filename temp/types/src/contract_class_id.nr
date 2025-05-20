use crate::constants::GENERATOR_INDEX__CONTRACT_LEAF;
use crate::traits::{Deserialize, Empty, FromField, Packable, Serialize, ToField};

pub struct ContractClassId {
    pub inner: Field,
}

impl Eq for ContractClassId {
    fn eq(self, other: ContractClassId) -> bool {
        other.inner == self.inner
    }
}

impl ToField for ContractClassId {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl FromField for ContractClassId {
    fn from_field(value: Field) -> Self {
        Self { inner: value }
    }
}

impl Serialize<1> for ContractClassId {
    fn serialize(self: Self) -> [Field; 1] {
        [self.inner]
    }
}

impl Deserialize<1> for ContractClassId {
    fn deserialize(fields: [Field; 1]) -> Self {
        Self { inner: fields[0] }
    }
}

impl Empty for ContractClassId {
    fn empty() -> Self {
        Self { inner: 0 }
    }
}

// Implement the Packable trait so ContractClassId can be stored in contract's storage.
impl Packable<1> for ContractClassId {
    fn pack(self) -> [Field; 1] {
        self.serialize()
    }

    fn unpack(fields: [Field; 1]) -> Self {
        Self::deserialize(fields)
    }
}

impl ContractClassId {
    pub fn compute(
        artifact_hash: Field,
        private_functions_root: Field,
        public_bytecode_commitment: Field,
    ) -> Self {
        let hash = crate::hash::poseidon2_hash_with_separator(
            [artifact_hash, private_functions_root, public_bytecode_commitment],
            GENERATOR_INDEX__CONTRACT_LEAF,
        ); // TODO(@spalladino): Update generator index
        ContractClassId::from_field(hash)
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}
