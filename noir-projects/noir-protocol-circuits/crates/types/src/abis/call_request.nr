use crate::address::AztecAddress;
use dep::std::cmp::Eq;
use crate::traits::Empty;
use crate::abis::caller_context::CallerContext;

struct CallRequest {
    hash: Field,
    caller_contract_address: AztecAddress,
    caller_context: CallerContext,
    start_side_effect_counter: u32,
    end_side_effect_counter: u32,
}

impl Eq for CallRequest {
    fn eq(self, call_request: CallRequest) -> bool {
        (call_request.hash == self.hash)
            & (call_request.caller_contract_address.eq(self.caller_contract_address))
            & (call_request.caller_context.eq(self.caller_context))
            & (call_request.start_side_effect_counter == self.start_side_effect_counter)
            & (call_request.end_side_effect_counter == self.end_side_effect_counter)
    }
}

impl Empty for CallRequest {
    fn empty() -> Self {
        CallRequest {
            hash: 0,
            caller_contract_address: AztecAddress::zero(),
            caller_context: CallerContext::empty(),
            start_side_effect_counter: 0,
            end_side_effect_counter: 0,
        }
    }
}

impl CallRequest {
    pub fn is_empty(self) -> bool {
        self.hash == 0
    }
}
