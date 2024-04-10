// docs:start:interfaces
use dep::aztec::prelude::{FunctionSelector, AztecAddress, EthAddress, PrivateContext, Deserialize};
use dep::aztec::context::{PublicContext, gas::GasOpts};

struct Token {
  address: AztecAddress,
}

impl Token {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn transfer_public(
        self: Self,
        context: &mut PublicContext,
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("transfer_public((Field),(Field),Field,Field)"),
            [from.to_field(), to.to_field(), amount, nonce],
            GasOpts::default()
        ).assert_empty();
    }

    pub fn unshield(
        self: Self,
        context: &mut PrivateContext,
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        let _ret = context.call_private_function(
            self.address,
            FunctionSelector::from_signature("unshield((Field),(Field),Field,Field)"),
            [from.to_field(), to.to_field(), amount, nonce]
        );
    }
}

struct TokenBridge {
  address: AztecAddress,
}

impl TokenBridge {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn token(self: Self, context: &mut PublicContext) -> AztecAddress {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("get_token()"),
            [],
            GasOpts::default()
        ).deserialize_into()
    }

    pub fn exit_to_l1_public(
        self: Self,
        context: &mut PublicContext,
        recipient: EthAddress,
        amount: Field,
        caller_on_l1: EthAddress,
        nonce: Field
    ) {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("exit_to_l1_public((Field),Field,(Field),Field)"),
            [recipient.to_field(), amount, caller_on_l1.to_field(), nonce],
            GasOpts::default()
        ).assert_empty();
    }
}
// docs:end:interfaces
