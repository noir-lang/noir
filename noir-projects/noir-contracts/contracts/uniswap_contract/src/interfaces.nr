// docs:start:interfaces
use dep::aztec::prelude::{FunctionSelector, AztecAddress, EthAddress, PrivateContext};
use dep::aztec::context::PublicContext;

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
        let _ = context.call_public_function(
            self.address,
            FunctionSelector::from_signature("transfer_public((Field),(Field),Field,Field)"),
            [from.to_field(), to.to_field(), amount, nonce]
        );
    }

    pub fn unshield(
        self: Self,
        context: &mut PrivateContext,
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        let _ = context.call_private_function(
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
        let return_values = context.call_public_function(
            self.address,
            FunctionSelector::from_signature("get_token()"),
            []
        );
        AztecAddress::from_field(return_values[0])
    }

    pub fn exit_to_l1_public(
        self: Self,
        context: &mut PublicContext,
        recipient: EthAddress,
        amount: Field,
        caller_on_l1: EthAddress,
        nonce: Field
    ) {
        let _ = context.call_public_function(
            self.address,
            FunctionSelector::from_signature("exit_to_l1_public((Field),Field,(Field),Field)"),
            [recipient.to_field(), amount, caller_on_l1.to_field(), nonce]
        );
    }
}
// docs:end:interfaces
