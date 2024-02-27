use dep::aztec::context::{PrivateContext, PublicContext};

use crate::asset::Asset;
use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, constants::RETURN_VALUES_LENGTH};

struct PriceFeed {
  address: AztecAddress,
}

impl PriceFeed {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn get_price(self: Self, context: PublicContext) -> U128 {
        let return_values = context.call_public_function(
            self.address,
            FunctionSelector::from_signature("get_price(Field)"),
            [0]
        );

        U128::from_integer(return_values[0])
    }
}

struct Token {
  address: AztecAddress,
}

impl Token {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn transfer_public(
        self: Self,
        context: PublicContext,
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("transfer_public((Field),(Field),Field,Field)"),
            [from.to_field(), to.to_field(), amount, nonce]
        );
    }

    pub fn mint_public(self: Self, context: PublicContext, to: AztecAddress, amount: Field) {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("mint_public((Field),Field)"),
            [to.to_field(), amount]
        );
    }

    pub fn burn_public(
        self: Self,
        context: PublicContext,
        from: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("burn_public((Field),Field,Field)"),
            [from.to_field(), amount, nonce]
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

    pub fn burn(
        self: Self,
        context: &mut PrivateContext,
        from: AztecAddress,
        amount: Field,
        nonce: Field
    ) -> [Field; RETURN_VALUES_LENGTH] {
        context.call_private_function(
            self.address,
            FunctionSelector::from_signature("burn((Field),Field,Field)"),
            [from.to_field(), amount, nonce]
        )
    }
}

struct Lending {
  address: AztecAddress,
}

impl Lending {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn update_accumulator(self: Self, context: PublicContext) -> Asset {
        let return_values = context.call_public_function_no_args(
            self.address,
            FunctionSelector::from_signature("update_accumulator()")
        );

        Asset::deserialize(return_values)
    }
}
