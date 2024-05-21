use dep::protocol_types::{
    transaction::tx_context::TxContext, abis::{call_context::CallContext, gas_settings::GasSettings},
    header::Header, traits::Empty
};

// PrivateContextInputs are expected to be provided to each private function
// docs:start:private-context-inputs
struct PrivateContextInputs {
    call_context : CallContext,
    historical_header: Header,
    tx_context: TxContext,
    start_side_effect_counter: u32,
}
// docs:end:private-context-inputs

impl Empty for PrivateContextInputs {
    fn empty() -> Self {
        PrivateContextInputs {
            call_context: CallContext::empty(),
            historical_header: Header::empty(),
            tx_context: TxContext::empty(),
            start_side_effect_counter: 0 as u32,
        }
    }
}
