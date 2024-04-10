use dep::aztec::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress};
use dep::aztec::prelude::Deserialize;
use dep::aztec::context::{PrivateContext, PublicContext, Context, gas::GasOpts};

struct SlowMap {
  address: AztecAddress,
}

impl SlowMap {
    pub fn at(address: AztecAddress) -> Self {
        Self { address }
    }

    pub fn initialize(self: Self, context: &mut PublicContext) {
        context.call_public_function_no_args(
            self.address,
            FunctionSelector::from_signature("initialize()")
        ).assert_empty();
    }

    pub fn read_at_pub(self: Self, context: &mut PublicContext, index: Field) -> Field {
        context.call_public_function(
            self.address,
            FunctionSelector::from_signature("read_at_pub(Field)"),
            [index],
            GasOpts::default()
        ).deserialize_into()
    }

    pub fn read_at(self: Self, context: &mut PrivateContext, index: Field) -> Field {
        context.call_private_function(
            self.address,
            FunctionSelector::from_signature("read_at(Field)"),
            [index]
        ).unpack_into()
    }

    pub fn update_at_private(self: Self, context: &mut PrivateContext, index: Field, new_value: Field) {
        let _ = context.call_private_function(
            self.address,
            FunctionSelector::from_signature("update_at_private(Field,Field)"),
            [index, new_value]
        );
    }
}
