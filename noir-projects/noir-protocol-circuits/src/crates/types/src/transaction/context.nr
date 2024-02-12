use crate::constants::GENERATOR_INDEX__TX_CONTEXT;
use crate::contrakt::deployment_data::ContractDeploymentData;
use crate::traits::Hash;

struct TxContext {
    is_fee_payment_tx : bool,
    is_rebate_payment_tx : bool,
    is_contract_deployment_tx : bool,

    contract_deployment_data : ContractDeploymentData,

    chain_id : Field,
    version : Field,
}

impl Hash for TxContext {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.is_fee_payment_tx as Field,
            self.is_rebate_payment_tx as Field,
            self.is_contract_deployment_tx as Field,
            self.contract_deployment_data.hash(),
            self.chain_id,
            self.version,
        ], GENERATOR_INDEX__TX_CONTEXT)
    }
}
