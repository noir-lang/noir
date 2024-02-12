use crate::abis::{
    function_data::FunctionData, private_circuit_public_inputs::PrivateCircuitPublicInputs,
    public_circuit_public_inputs::PublicCircuitPublicInputs
};
use crate::address::AztecAddress;
use crate::constants::{GENERATOR_INDEX__CONTRACT_LEAF};
use crate::traits::Hash;

struct ContractClassId {
  inner: Field
}

impl Eq for ContractClassId {
    fn eq(self, other: ContractClassId) -> bool {
        other.inner == self.inner
    }
}

impl ContractClassId {
    pub fn compute(
        artifact_hash: Field,
        private_functions_root: Field,
        public_bytecode_commitment: Field
    ) -> Self {
        let hash = dep::std::hash::pedersen_hash_with_separator(
            [
            artifact_hash,
            private_functions_root,
            public_bytecode_commitment
        ],
            GENERATOR_INDEX__CONTRACT_LEAF
        ); // TODO(@spalladino): Update generator index

        ContractClassId::from_field(hash)
    }

    fn to_field(self) -> Field {
        self.inner as Field
    }

    pub fn from_field(value: Field) -> Self {
        Self { inner: value }
    }

    pub fn serialize(self: Self) -> [Field; 1] {
        [self.inner]
    }

    pub fn deserialize(fields: [Field; 1]) -> Self {
        Self { inner: fields[0] }
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}
