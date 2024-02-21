use dep::aztec::protocol_types::{
    contract_class_id::ContractClassId,
    constants::{MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS, REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE},
    traits::{Serialize}
};

// #[event]
struct ContractClassRegistered {
    contract_class_id: ContractClassId,
    version: Field,
    artifact_hash: Field,
    private_functions_root: Field,
    packed_public_bytecode: [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS],
}

impl Serialize<MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS + 5> for ContractClassRegistered {
    fn serialize(self: Self) -> [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS + 5] {
        let mut packed = [0; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS + 5];
        packed[0] = REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE;
        packed[1] = self.contract_class_id.to_field();
        packed[2] = self.version;
        packed[3] = self.artifact_hash;
        packed[4] = self.private_functions_root;
        for i in 0..MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS {
            packed[i + 5] = self.packed_public_bytecode[i];
        }
        packed
    }
}
