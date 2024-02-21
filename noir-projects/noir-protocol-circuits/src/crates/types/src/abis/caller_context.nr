use crate::address::AztecAddress;
use dep::std::cmp::Eq;
use crate::traits::Empty;

struct CallerContext {
    msg_sender: AztecAddress,
    storage_contract_address: AztecAddress,
}

impl Eq for CallerContext {
    fn eq(self, caller_context: CallerContext) -> bool {
        caller_context.msg_sender.eq(self.msg_sender)
            & caller_context.storage_contract_address.eq(self.storage_contract_address)
    }
}

impl Empty for CallerContext {
    fn empty() -> Self {
        CallerContext {
            msg_sender: AztecAddress::zero(),
            storage_contract_address: AztecAddress::zero(),
        }
    }
}

impl CallerContext {
    pub fn is_empty(self) -> bool {
        self.msg_sender.is_zero() & self.storage_contract_address.is_zero()
    }
}
