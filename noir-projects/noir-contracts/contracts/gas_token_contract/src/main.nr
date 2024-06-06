mod lib;

contract GasToken {
    use dep::aztec::{
        protocol_types::{
        contract_class_id::ContractClassId, abis::function_selector::FunctionSelector,
        address::{AztecAddress, EthAddress},
        constants::{DEPLOYER_CONTRACT_ADDRESS, REGISTERER_CONTRACT_ADDRESS}
    },
        state_vars::{SharedImmutable, PublicMutable, Map},
        oracle::get_contract_instance::get_contract_instance, deploy::deploy_contract
    };

    use dep::deployer::ContractInstanceDeployer;
    use dep::registerer::ContractClassRegisterer;

    use crate::lib::{calculate_fee, get_bridge_gas_msg_hash};

    #[aztec(storage)]
    struct Storage {
        // This map is accessed directly by protocol circuits to check balances for fee payment.
        // Do not change this storage layout unless you also update the base rollup circuits.
        balances: Map<AztecAddress, PublicMutable<U128>>,
        portal_address: SharedImmutable<EthAddress>,
    }

    // Not flagged as initializer to reduce cost of checking init nullifier in all functions.
    // This function should be called as entrypoint to initialize the contract by minting itself funds.
    #[aztec(private)]
    fn deploy(
        artifact_hash: Field,
        private_functions_root: Field,
        public_bytecode_commitment: Field,
        portal_address: EthAddress
    ) {
        // Validate contract class parameters are correct
        let self = context.this_address();
        let instance = get_contract_instance(self);
        let contract_class_id = ContractClassId::compute(
            artifact_hash,
            private_functions_root,
            public_bytecode_commitment
        );
        assert(
            instance.contract_class_id == contract_class_id, "Invalid contract class id computed for gas token"
        );

        // Increase self balance and set as fee payer, and end setup
        let deploy_fees = 20000000000;
        GasToken::at(self)._increase_public_balance(self, deploy_fees).enqueue(&mut context);
        context.set_as_fee_payer();
        context.end_setup();

        // Register class and publicly deploy contract
        let _register = ContractClassRegisterer::at(AztecAddress::from_field(REGISTERER_CONTRACT_ADDRESS)).register(
            artifact_hash,
            private_functions_root,
            public_bytecode_commitment
        ).call(&mut context);
        let _deploy = ContractInstanceDeployer::at(AztecAddress::from_field(DEPLOYER_CONTRACT_ADDRESS)).deploy(
            instance.salt,
            instance.contract_class_id,
            instance.initialization_hash,
            instance.public_keys_hash,
            true
        ).call(&mut context);

        // Enqueue call to set the portal address
        GasToken::at(self).set_portal(portal_address).enqueue(&mut context);
    }

    // We purposefully not set this function as an initializer so we do not bind
    // the contract to a specific L1 portal address, since the gas token address
    // is a hardcoded constant in the rollup circuits.
    #[aztec(public)]
    fn set_portal(portal_address: EthAddress) {
        assert(storage.portal_address.read_public().is_zero());
        storage.portal_address.initialize(portal_address);
    }

    #[aztec(private)]
    fn claim(to: AztecAddress, amount: Field, secret: Field) {
        let content_hash = get_bridge_gas_msg_hash(to, amount);
        let portal_address = storage.portal_address.read_private();
        assert(!portal_address.is_zero());

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content_hash, secret, portal_address);

        // TODO(palla/gas) Emit an unencrypted log to announce which L1 to L2 message has been claimed
        // Otherwise, we cannot trace L1 deposits to their corresponding claims on L2

        GasToken::at(context.this_address())._increase_public_balance(to, amount).enqueue(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _increase_public_balance(to: AztecAddress, amount: Field) {
        let new_balance = storage.balances.at(to).read().add(U128::from_integer(amount));
        storage.balances.at(to).write(new_balance);
    }

    // TODO(palla/gas) Remove this function and use the private claim flow only
    #[aztec(public)]
    fn claim_public(to: AztecAddress, amount: Field, secret: Field, leaf_index: Field) {
        let content_hash = get_bridge_gas_msg_hash(to, amount);
        let portal_address = storage.portal_address.read_public();
        assert(!portal_address.is_zero());

        // Consume message and emit nullifier
        context.consume_l1_to_l2_message(content_hash, secret, portal_address, leaf_index);

        let new_balance = storage.balances.at(to).read() + U128::from_integer(amount);
        storage.balances.at(to).write(new_balance);
    }

    // TODO(@just-mitch): remove this function before mainnet deployment
    // convenience function for testing
    // the true canonical gas token contract will not have this function
    #[aztec(public)]
    fn mint_public(to: AztecAddress, amount: Field) {
        let amount = U128::from_integer(amount);
        let new_balance = storage.balances.at(to).read().add(amount);

        storage.balances.at(to).write(new_balance);
    }

    #[aztec(public)]
    #[aztec(view)]
    fn check_balance(fee_limit: Field) {
        let fee_limit = U128::from_integer(fee_limit);
        assert(storage.balances.at(context.msg_sender()).read() >= fee_limit, "Balance too low");
    }

    // utility function for testing
    #[aztec(public)]
    #[aztec(view)]
    fn balance_of_public(owner: AztecAddress) -> pub Field {
        storage.balances.at(owner).read().to_field()
    }
}
