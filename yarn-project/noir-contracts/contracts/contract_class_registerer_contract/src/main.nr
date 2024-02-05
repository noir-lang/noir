contract ContractClassRegisterer {
    use dep::std::option::Option;
    use dep::aztec::protocol_types::{
        address::{ AztecAddress, EthAddress },
        contract_class::ContractClassId,
        constants::{MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS, CONTRACT_CLASS_REGISTERED_MAGIC_VALUE}
    };

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3590): Remove this once the issue is fixed
    use dep::aztec::protocol_types;

    use dep::aztec::log::{ emit_unencrypted_log, emit_unencrypted_log_from_private};

    #[event]
    struct ContractClassRegistered {
        contract_class_id: ContractClassId,
        version: Field,
        artifact_hash: Field,
        private_functions_root: Field,
        packed_public_bytecode: [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS],
    }

    impl ContractClassRegistered {
        fn serialize(self: Self) -> [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS + 5] {
            let mut packed = [0; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS + 5];
            packed[0] = CONTRACT_CLASS_REGISTERED_MAGIC_VALUE;
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

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn register(
        artifact_hash: Field,
        private_functions_root: Field,
        public_bytecode_commitment: Field,
        packed_public_bytecode: [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS]
    ) {
        // TODO: Validate public_bytecode_commitment is the correct commitment of packed_public_bytecode
        // TODO: Validate packed_public_bytecode is legit public bytecode

        // Compute contract class id from preimage
        let contract_class_id = ContractClassId::compute(
            artifact_hash,
            private_functions_root,
            public_bytecode_commitment
        );

        // Emit the contract class id as a nullifier to be able to prove that this class has been (not) registered
        let event = ContractClassRegistered { contract_class_id, version: 1, artifact_hash, private_functions_root, packed_public_bytecode };
        context.push_new_nullifier(contract_class_id.to_field(), 0);

        // Broadcast class info including public bytecode
        let event_payload = event.serialize();
        dep::aztec::oracle::debug_log::debug_log_array_with_prefix("ContractClassRegistered", event_payload);
        emit_unencrypted_log_from_private(&mut context, event_payload);
    }
}
