use crate::{
    contrakt::deployment_data::ContractDeploymentData,
    tests::fixtures,
    transaction::context::TxContext,
};

pub fn build_contract_deployment_data(is_constructor: bool) -> ContractDeploymentData {
    let mut contract_deployment_data: ContractDeploymentData = dep::std::unsafe::zeroed();
    if is_constructor {
        let contract_data = fixtures::contracts::default_contract;
        let constructor = fixtures::contract_functions::default_constructor;
        contract_deployment_data = ContractDeploymentData {
            deployer_public_key: fixtures::DEPLOYER_PUBLIC_KEY,
            constructor_vk_hash: constructor.vk_hash,
            function_tree_root: contract_data.function_tree_root,
            contract_address_salt: contract_data.contract_address_salt,
            portal_contract_address: contract_data.portal_contract_address,
        };
    }
    contract_deployment_data
}

pub fn build_tx_context(is_constructor: bool) -> TxContext {
    let contract_deployment_data = build_contract_deployment_data(is_constructor);
    TxContext {
        is_fee_payment_tx: false,
        is_rebate_payment_tx: false,
        is_contract_deployment_tx: is_constructor,
        contract_deployment_data,
        chain_id: 1,
        version: 0
    }
}
