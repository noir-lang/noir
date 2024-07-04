contract ContractInstanceDeployer {
    use dep::aztec::protocol_types::{
        address::{AztecAddress, EthAddress, PublicKeysHash, PartialAddress},
        contract_class_id::ContractClassId, constants::DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE,
        traits::Serialize, abis::log_hash::LogHash
    };
    use dep::aztec::{
        context::PrivateContext, hash::compute_unencrypted_log_hash,
        oracle::logs::emit_unencrypted_log_private_internal
    };

    // @todo This should be using an event, but currently we only support fields in the struct.
    // #[aztec(event)]
    struct ContractInstanceDeployed {
        address: AztecAddress,
        version: u8,
        salt: Field, 
        contract_class_id: ContractClassId, 
        initialization_hash: Field, 
        public_keys_hash: PublicKeysHash, 
        deployer: AztecAddress,
    }

    global CONTRACT_INSTANCE_DEPLOYED_SERIALIZED_SIZE: Field = 8;

    impl Serialize<CONTRACT_INSTANCE_DEPLOYED_SERIALIZED_SIZE> for ContractInstanceDeployed {
        fn serialize(self: Self) -> [Field; CONTRACT_INSTANCE_DEPLOYED_SERIALIZED_SIZE] {
            [
                DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE,
                self.address.to_field(),
                self.version as Field,
                self.salt,
                self.contract_class_id.to_field(),
                self.initialization_hash,
                self.public_keys_hash.to_field(),
                self.deployer.to_field(),
            ]
        }
    }

    #[aztec(private)]
    fn deploy(
        salt: Field,
        contract_class_id: ContractClassId,
        initialization_hash: Field,
        public_keys_hash: PublicKeysHash,
        universal_deploy: bool
    ) {
        // TODO(@spalladino): assert nullifier_exists silo(contract_class_id, ContractClassRegisterer)

        let deployer = if universal_deploy {
            AztecAddress::zero()
        } else {
            context.msg_sender()
        };

        let partial_address = PartialAddress::compute(contract_class_id, salt, initialization_hash, deployer);

        let address = AztecAddress::compute(public_keys_hash, partial_address);

        // Emit the address as a nullifier to be able to prove that this instance has been (not) deployed
        context.push_nullifier(address.to_field(), 0);

        // Broadcast the event
        let event = ContractInstanceDeployed { contract_class_id, address, public_keys_hash, initialization_hash, salt, deployer, version: 1 };

        let payload = event.serialize();
        dep::aztec::oracle::debug_log::debug_log_format("ContractInstanceDeployed: {}", payload);

        let contract_address = context.this_address();
        let counter = context.next_counter();

        // @todo This is very inefficient, we are doing a lot of back and forth conversions.
        let log_slice = payload.to_be_bytes_arr();
        let log_hash = compute_unencrypted_log_hash(contract_address, payload);
        // 40 = addr (32) + raw log len (4) + processed log len (4)
        let len = 40 + log_slice.len().to_field();
        let side_effect = LogHash { value: log_hash, counter, length: len };
        context.unencrypted_logs_hashes.push(side_effect);

        let _void = emit_unencrypted_log_private_internal(contract_address, payload, counter);
    }
}
