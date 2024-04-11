use crate::context::globals::public_global_variables::PublicGlobalVariables;

use dep::protocol_types::{abis::call_context::CallContext, header::Header, traits::Empty};

// PublicContextInputs are expected to be provided to each public function
// docs:start:public-context-inputs
struct PublicContextInputs {
    call_context: CallContext,
    historical_header: Header,

    public_global_variables: PublicGlobalVariables,

    start_side_effect_counter: u32,
}
// docs:end:public-context-inputs

impl Empty for PublicContextInputs {
    fn empty() -> Self {
        PublicContextInputs {
            call_context: CallContext::empty(),
            historical_header: Header::empty(),
            public_global_variables: PublicGlobalVariables::empty(),
            start_side_effect_counter: 0 as u32,
        }
    }
}