mod storage;

use dep::aztec::context::{
    PrivateContext,
    PublicContext
};

use crate::storage::Asset;
use dep::aztec::constants_gen::RETURN_VALUES_LENGTH;

struct PriceFeed {
  address: Field,
}

impl PriceFeed {
  fn at(address: Field) -> Self {
    Self { address }
  }

  fn get_price(self: Self, context: PublicContext) -> u120 {
    let return_values = context.call_public_function(
      self.address, 
      3359284436,
      [0]
    );

    return_values[0] as u120
  }
}

struct Token {
  address: Field,
}

impl Token {
  fn at(address: Field) -> Self {
    Self { address }
  }

  fn transfer_pub(self: Self, context: PublicContext, to: Field, amount: Field) {
    let _transfer_return_values = context.call_public_function(
      self.address,
      1012824788, 
      [to, amount]
    );
  }

  fn transfer_from_pub(self: Self, context: PublicContext, from: Field, to: Field, amount: Field) {
    let _transfer_return_values = context.call_public_function(
      self.address, 
      1602017294, 
      [from, to, amount]
    );
  }

  fn owner_mint_pub(self: Self, context: PublicContext, to: Field, amount: Field) {
    let _transfer_return_values = context.call_public_function(
      self.address, 
      1071038680, 
      [to, amount]
    );
  }

  // Private
  fn unshield(self: Self, context: &mut PrivateContext, from: Field, to: Field, amount: Field) -> [Field; RETURN_VALUES_LENGTH] {
    context.call_private_function(
      self.address,
      2423803924, 
      [from, to, amount]
    )
  }
}

struct Lending {
  address: Field,
}

impl Lending {
  fn at(address: Field) -> Self {
    Self { address }
  }

  fn update_accumulator(self: Self, context: PublicContext) -> Asset {
    let return_values = context.call_public_function_no_args(
      self.address, 
      0x1873b536
    );

    Asset {
      interest_accumulator: return_values[0] as u120,
      last_updated_ts: return_values[1] as u120,
      loan_to_value: return_values[2] as u120,
      oracle_address: return_values[3],
    }
  }
}