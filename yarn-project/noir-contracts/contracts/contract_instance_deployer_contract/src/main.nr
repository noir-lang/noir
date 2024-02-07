mod events;

contract ContractInstanceDeployer {
    use dep::std::option::Option;
    use dep::aztec::protocol_types::{
        address::{ AztecAddress, EthAddress, PublicKeysHash, PartialAddress },
        contract_class::ContractClassId,
        constants::{DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE},
        traits::{Serialize}
    };

    use dep::aztec::log::{ emit_unencrypted_log, emit_unencrypted_log_from_private};

    use crate::events::{
        instance_deployed::ContractInstanceDeployed,
    };

    #[aztec(private)]
    fn constructor() {}

    #[aztec(private)]
    fn deploy(
        salt: Field,
        contract_class_id: ContractClassId,
        initialization_hash: Field,
        portal_contract_address: EthAddress,
        public_keys_hash: PublicKeysHash,
        universal_deploy: bool
    ) {
        // TODO(@spalladino): assert nullifier_exists silo(contract_class_id, ContractClassRegisterer)
        // TODO(@spalladino): assert is_valid_eth_address(portal_contract_address)

        // TODO(#4434) Add deployer field to instance calculation
        // let deployer = if universal_deploy { Field::zero() } else { context.msg_sender() };

        let partial_address = PartialAddress::compute(
            contract_class_id,
            salt,
            initialization_hash,
            portal_contract_address
        );

        let address = AztecAddress::compute(public_keys_hash, partial_address);

        // Emit the address as a nullifier to be able to prove that this instance has been (not) deployed
        context.push_new_nullifier(address.to_field(), 0);

        // Broadcast the event
        let event = ContractInstanceDeployed {
            contract_class_id,
            address,
            public_keys_hash,
            portal_contract_address,
            initialization_hash,
            salt,
            universal_deploy,
            version: 1
        };
        let event_payload = event.serialize();
        dep::aztec::oracle::debug_log::debug_log_array_with_prefix("ContractInstanceDeployed", event_payload);
        emit_unencrypted_log_from_private(&mut context, event_payload);
    }
}
