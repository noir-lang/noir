use crate::address::AztecAddress;

struct CallerContext {
    msg_sender: AztecAddress,
    storage_contract_address: AztecAddress,
}

impl CallerContext {
    pub fn empty() -> Self {
        CallerContext {
            msg_sender: AztecAddress::zero(),
            storage_contract_address: AztecAddress::zero(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.msg_sender.is_zero() & self.storage_contract_address.is_zero()
    }

    pub fn eq(self, caller_context: CallerContext) -> bool {
        caller_context.msg_sender.eq(self.msg_sender)
            & caller_context.storage_contract_address.eq(self.storage_contract_address)
    }
}

struct CallRequest {
    hash: Field,
    caller_contract_address: AztecAddress,
    caller_context: CallerContext,
}

impl CallRequest {
    pub fn empty() -> Self {
        Self {
            hash: 0,
            caller_contract_address: AztecAddress::zero(),
            caller_context: dep::std::unsafe::zeroed(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.hash == 0
    }

    pub fn eq(self, call_request: CallRequest) -> bool {
        (call_request.hash == self.hash)
            & (call_request.caller_contract_address.eq(self.caller_contract_address))
            & (call_request.caller_context.eq(self.caller_context))
    }
}