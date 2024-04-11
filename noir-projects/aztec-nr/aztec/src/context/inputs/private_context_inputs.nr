use dep::protocol_types::{abis::call_context::CallContext, header::Header, traits::Empty};
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

impl Empty for PrivateContextInputs {
    fn empty() -> Self {
        PrivateContextInputs {
            call_context : CallContext::empty(),
            historical_header: Header::empty(),
            private_global_variables: PrivateGlobalVariables::empty(),
            start_side_effect_counter: 0 as u32,
        }
    }
}
