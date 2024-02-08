use dep::protocol_types::{
    abis::call_context::CallContext,
    contrakt::deployment_data::ContractDeploymentData,
    header::Header,
};
use crate::context::globals::private_global_variables::PrivateGlobalVariables;

// PrivateContextInputs are expected to be provided to each private function
// docs:start:private-context-inputs
struct PrivateContextInputs {
    call_context : CallContext,
    historical_header: Header,
    contract_deployment_data: ContractDeploymentData,
    private_global_variables: PrivateGlobalVariables,
}
// docs:end:private-context-inputs