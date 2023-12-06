use crate::abis::function_selector::FunctionSelector;
use crate::address::{EthAddress,Address};
use crate::constants::GENERATOR_INDEX__CALL_CONTEXT;

struct CallContext{
    msg_sender : Address,
    storage_contract_address : Address,
    portal_contract_address : EthAddress,

    function_selector : FunctionSelector,

    is_delegate_call : bool,
    is_static_call : bool,
    is_contract_deployment : bool,
}

impl CallContext {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.msg_sender.to_field(),
            self.storage_contract_address.to_field(),
            self.portal_contract_address.to_field(),
            self.function_selector.to_field(),
            self.is_delegate_call as Field,
            self.is_static_call as Field,
            self.is_contract_deployment as Field,
        ], GENERATOR_INDEX__CALL_CONTEXT)
    }

    fn assert_is_zero(self) {
        assert(self.msg_sender.to_field() == 0);
        assert(self.storage_contract_address.to_field() == 0);
        assert(self.portal_contract_address.to_field() == 0);
        assert(self.function_selector.to_field() == 0);
        assert(self.is_delegate_call == false);
        assert(self.is_static_call == false);
        assert(self.is_contract_deployment == false);
    }

    fn eq(self, call_context: CallContext) -> bool {
        call_context.msg_sender.eq(self.msg_sender)
            & call_context.storage_contract_address.eq(self.storage_contract_address)
            & call_context.portal_contract_address.eq(self.portal_contract_address)
            & call_context.function_selector.eq(self.function_selector)
            & (call_context.is_delegate_call == self.is_delegate_call)
            & (call_context.is_static_call == self.is_static_call)
            & (call_context.is_contract_deployment == self.is_contract_deployment)
    }
}
