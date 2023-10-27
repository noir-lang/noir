use crate::abis::function_selector::FunctionSelector;
use crate::address::{EthAddress,Address};
use dep::aztec::constants_gen;

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
        ], constants_gen::GENERATOR_INDEX__CALL_CONTEXT)
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
}
