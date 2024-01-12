// docs:start:interfaces
use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector,
        address::{
          AztecAddress,
          EthAddress,
        },
    };
use dep::aztec::{
  context::{ PrivateContext, PublicContext, Context },
};

struct Token {
  address: AztecAddress,
}

impl Token {
  pub fn at(address: AztecAddress) -> Self {
    Self { address }
  }

  pub fn transfer_public(self: Self, context: PublicContext, from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) {
    context.call_public_function(
      self.address,
      FunctionSelector::from_signature("transfer_public((Field),(Field),Field,Field)"),
      [from.to_field(), to.to_field(), amount, nonce]
    );
  }

  pub fn unshield(self: Self, context: &mut PrivateContext, from: AztecAddress, to: AztecAddress,  amount: Field, nonce: Field) {
    context.call_private_function(
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

  pub fn token(self: Self, context: PublicContext) -> AztecAddress {
    let return_values = context.call_public_function(self.address, FunctionSelector::from_signature("get_token()"), []);
    AztecAddress::from_field(return_values[0])
  }

  pub fn exit_to_l1_public(self: Self, context: PublicContext, recipient: EthAddress, amount: Field, callerOnL1: EthAddress, nonce: Field) {
    context.call_public_function(
      self.address,
      FunctionSelector::from_signature("exit_to_l1_public((Field),Field,(Field),Field)"),
      [recipient.to_field(), amount, callerOnL1.to_field(), nonce]
    );
  }
}
// docs:end:interfaces
