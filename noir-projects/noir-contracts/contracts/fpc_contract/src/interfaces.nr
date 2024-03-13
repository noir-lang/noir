use dep::aztec::prelude::{AztecAddress, EthAddress, FunctionSelector, PrivateContext};
use dep::aztec::context::PublicContext;

use dep::aztec::protocol_types::constants::RETURN_VALUES_LENGTH;

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

    pub fn shield(
        self: Self,
        context: &mut PublicContext,
        from: AztecAddress,
        amount: Field,
        secret_hash: Field,
        nonce: Field
    ) {
        let _ = context.call_public_function(
            self.address,
            FunctionSelector::from_signature("shield((Field),Field,Field,Field)"),
            [from.to_field(), amount, secret_hash, nonce]
        );
    }

    // Private
    pub fn unshield(
        self: Self,
        context: &mut PrivateContext,
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field
    ) -> [Field; RETURN_VALUES_LENGTH] {
        context.call_private_function(
            self.address,
            FunctionSelector::from_signature("unshield((Field),(Field),Field,Field)"),
            [from.to_field(), to.to_field(), amount, nonce]
        )
    }
}
