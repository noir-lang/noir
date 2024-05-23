use crate::context::{PrivateContext, PublicContext};
use dep::protocol_types::address::AztecAddress;
use dep::std::test::OracleMock;

struct ContextBuilder {
    block_number: Option<Field>,
    contract_address: Option<AztecAddress>,
}

impl ContextBuilder {
    fn new() -> Self {
        Self { block_number: Option::none(), contract_address: Option::none() }
    }

    fn block_number(&mut self, block_number: Field) -> &mut Self {
        self.block_number = Option::some(block_number);
        self
    }

    fn contract_address(&mut self, contract_address: AztecAddress) -> &mut Self {
        self.contract_address = Option::some(contract_address);
        self
    }

    fn private(&mut self) -> PrivateContext {
        let mut context = PrivateContext::empty();

        if self.block_number.is_some() {
            context.inputs.historical_header.global_variables.block_number = self.block_number.unwrap_unchecked();
        }

        if self.contract_address.is_some() {
            context.inputs.call_context.storage_contract_address = self.contract_address.unwrap_unchecked();
        }

        context
    }

    fn public(&mut self) -> PublicContext {
        let mut context = PublicContext::empty();

        if self.block_number.is_some() {
            let _ = OracleMock::mock("avmOpcodeBlockNumber").returns(self.block_number.unwrap());
        }

        if self.contract_address.is_some() {
            let _ = OracleMock::mock("avmOpcodeAddress").returns(self.contract_address.unwrap());
        }

        context
    }
}
