use crate::{
    contrakt::contract_deployment_data::ContractDeploymentData, tests::fixtures,
    transaction::tx_context::TxContext, address::compute_initialization_hash
};

pub fn build_contract_deployment_data(is_constructor: bool, args_hash: Field) -> ContractDeploymentData {
    let mut contract_deployment_data: ContractDeploymentData = dep::std::unsafe::zeroed();
    if is_constructor {
        let contract_data = fixtures::contracts::default_contract;
        let constructor = fixtures::contract_functions::default_constructor;
        contract_deployment_data = ContractDeploymentData {
            public_key: fixtures::PUBLIC_KEY,
            initialization_hash: compute_initialization_hash(constructor.data.selector.to_field(), args_hash),
            contract_class_id: contract_data.contract_class_id,
            contract_address_salt: contract_data.contract_address_salt,
            portal_contract_address: contract_data.portal_contract_address,
        };
    }
    contract_deployment_data
}

pub fn build_tx_context(is_constructor: bool, args_hash: Field) -> TxContext {
    let contract_deployment_data = build_contract_deployment_data(is_constructor, args_hash);
    TxContext {
        is_fee_payment_tx: false,
        is_rebate_payment_tx: false,
        is_contract_deployment_tx: is_constructor,
        contract_deployment_data,
        chain_id: 1,
        version: 0
    }
}
