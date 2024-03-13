use dep::protocol_types::{
    abis::call_context::CallContext,
    header::Header
};
use crate::context::globals::private_global_variables::PrivateGlobalVariables;

// PrivateContextInputs are expected to be provided to each private function
// docs:start:private-context-inputs
struct PrivateContextInputs {
    call_context : CallContext,
    historical_header: Header,
    private_global_variables: PrivateGlobalVariables,
    start_side_effect_counter: u32,
}
// docs:end:private-context-inputs
