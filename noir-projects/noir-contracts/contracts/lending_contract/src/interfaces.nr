use dep::aztec::prelude::{FunctionSelector, AztecAddress, PrivateContext, Deserialize};

use dep::aztec::context::{PublicContext, gas::GasOpts};

use crate::asset::Asset;

struct PriceFeed {
  address: AztecAddress,
}

impl PriceFeed {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn get_price(self: Self, context: &mut PublicContext) -> U128 {
        let price_field: Field = context.call_public_function(
            self.address,
            FunctionSelector::from_signature("get_price(Field)"),
            [0],
            GasOpts::default()
        ).deserialize_into();

        U128::from_integer(price_field)
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

    pub fn mint_public(self: Self, context: &mut PublicContext, to: AztecAddress, amount: Field) {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("mint_public((Field),Field)"),
            [to.to_field(), amount],
            GasOpts::default()
        ).assert_empty();
    }

    pub fn burn_public(
        self: Self,
        context: &mut PublicContext,
        from: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("burn_public((Field),Field,Field)"),
            [from.to_field(), amount, nonce],
            GasOpts::default()
        ).assert_empty();
    }

    // Private
    pub fn unshield(
        self: Self,
        context: &mut PrivateContext,
        from: AztecAddress,
        to: AztecAddress,
        amount: Field,
        nonce: Field
    ) {
        context.call_private_function(
            self.address,
            FunctionSelector::from_signature("unshield((Field),(Field),Field,Field)"),
            [from.to_field(), to.to_field(), amount, nonce]
        );
    }

    pub fn burn(self: Self, context: &mut PrivateContext, from: AztecAddress, amount: Field, nonce: Field) {
        context.call_private_function(
            self.address,
            FunctionSelector::from_signature("burn((Field),Field,Field)"),
            [from.to_field(), amount, nonce]
        );
    }
}

struct Lending {
  address: AztecAddress,
}

impl Lending {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn update_accumulator(self: Self, context: &mut PublicContext) -> Asset {
        context.call_public_function_no_args(
            self.address,
            FunctionSelector::from_signature("update_accumulator()")
        ).deserialize_into()
    }
}
