use crate::{
    abis::function_selector::FunctionSelector,
    address::{EthAddress,AztecAddress},
    constants::{
        CALL_CONTEXT_LENGTH,
        GENERATOR_INDEX__CALL_CONTEXT,
    },
    hash::pedersen_hash,
};

// docs:start:call-context
struct CallContext {
    msg_sender : AztecAddress,
    storage_contract_address : AztecAddress,
    portal_contract_address : EthAddress,

    function_selector : FunctionSelector,

    is_delegate_call : bool,
    is_static_call : bool,
    is_contract_deployment : bool,
}
// docs:end:call-context

impl CallContext {
    fn serialize(self) -> [Field; CALL_CONTEXT_LENGTH] {
        [
            self.msg_sender.to_field(),
            self.storage_contract_address.to_field(),
            self.portal_contract_address.to_field(),
            self.function_selector.to_field(),
            self.is_delegate_call as Field,
            self.is_static_call as Field,
            self.is_contract_deployment as Field,
        ]
    }

    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__CALL_CONTEXT)
    }

    fn assert_is_zero(self) {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3595)
        assert(self.msg_sender.to_field() == 0);
        assert(self.storage_contract_address.to_field() == 0);
        assert(self.portal_contract_address.to_field() == 0);
        assert(self.function_selector.to_field() == 0);
        assert(self.is_delegate_call == false);
        assert(self.is_static_call == false);
        assert(self.is_contract_deployment == false);
    }

    fn eq(self, call_context: CallContext) -> bool {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3595)
        call_context.msg_sender.eq(self.msg_sender)
            & call_context.storage_contract_address.eq(self.storage_contract_address)
            & call_context.portal_contract_address.eq(self.portal_contract_address)
            & call_context.function_selector.eq(self.function_selector)
            & (call_context.is_delegate_call == self.is_delegate_call)
            & (call_context.is_static_call == self.is_static_call)
            & (call_context.is_contract_deployment == self.is_contract_deployment)
    }
}
