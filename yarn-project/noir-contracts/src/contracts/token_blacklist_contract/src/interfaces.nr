use dep::aztec::{
  context::{ PrivateContext, PublicContext, Context },
  selector::compute_selector,
  types::address::AztecAddress,
};

struct SlowMap {
  address: Field,
}

impl SlowMap {
  pub fn at(address: AztecAddress) -> Self {
    Self { address: address.address }
  }

  pub fn initialize(self: Self, context: PublicContext) {
    let _return_values = context.call_public_function_no_args(
      self.address,
      compute_selector("initialize()")
    );
  }

  pub fn read_at_pub(self: Self, context: PublicContext, index: Field) -> Field {
    let _return_values = context.call_public_function(
      self.address,
      compute_selector("read_at_pub(Field)"),
      [index]
    );
    _return_values[0]
  }

  pub fn read_at(self: Self, context: &mut PrivateContext, index: Field) -> Field {
    let _return_values = context.call_private_function(
      self.address,
      compute_selector("read_at(Field)"),
      [index]
    );
    _return_values[0]
  }

  pub fn update_at_private(self: Self, context: &mut PrivateContext, index: Field, new_value: Field) {
    let _return_values = context.call_private_function(
      self.address,
      compute_selector("update_at_private(Field,Field)"),
      [index, new_value]
    );
  }
}