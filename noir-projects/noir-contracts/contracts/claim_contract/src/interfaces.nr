use dep::aztec::{
    protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress},
    context::PrivateContext
};

struct Token {
  address: AztecAddress,
}

impl Token {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    fn mint_public(self: Self, context: &mut PrivateContext, to: AztecAddress, amount: Field) {
        let _ret = context.call_public_function(
            self.address,
            FunctionSelector::from_signature("mint_public((Field),Field)"),
            [to.to_field(), amount]
        );
    }

    pub fn transfer(
        self: Self,
        context: &mut PrivateContext,
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        let _ret = context.call_private_function(
            self.address,
            FunctionSelector::from_signature("transfer((Field),(Field),Field,Field)"),
            [from.to_field(), to.to_field(), amount, nonce]
        );
    }
}
