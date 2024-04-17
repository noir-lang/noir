use dep::protocol_types::{hash::pedersen_hash, traits::FromField, address::AztecAddress};

use crate::context::{PrivateContext, Context};
use crate::history::public_storage::public_storage_historical_read;
use crate::public_storage;
use crate::state_vars::{storage::Storage, shared_mutable::scheduled_value_change::ScheduledValueChange};

struct SharedMutablePrivateGetter<T, DELAY> {
  context: PrivateContext,
  // The contract address of the contract we want to read from
  other_contract_address: AztecAddress,
  // The storage slot where the SharedMutable is stored on the other contract
  storage_slot: Field,
}

// We have this as a view-only interface to reading Shared Mutables in other contracts.
// Currently the Shared Mutable does not support this. We can adapt SharedMutable at a later date
impl<T, DELAY> SharedMutablePrivateGetter<T, DELAY> {
  pub fn new(
    context: PrivateContext,
    other_contract_address: AztecAddress,
    storage_slot: Field,
  ) -> Self {
    assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
    assert(other_contract_address.to_field() != 0, "Other contract address cannot be 0");
    Self { context, other_contract_address, storage_slot }
  }

  pub fn get_current_value_in_private(self) -> T where T: FromField {
    let mut context = self.context;

    let (scheduled_value_change, historical_block_number) = self.historical_read_from_public_storage(context);
    let block_horizon = scheduled_value_change.get_block_horizon(historical_block_number);

    // We prevent this transaction from being included in any block after the block horizon, ensuring that the 
    // historical public value matches the current one, since it can only change after the horizon.
    context.set_tx_max_block_number(block_horizon);
    scheduled_value_change.get_current_at(historical_block_number)
  }

  fn historical_read_from_public_storage(
      self,
      context: PrivateContext
  ) -> (ScheduledValueChange<T, DELAY>, u32) where T: FromField {
    let derived_slot = self.get_derived_storage_slot();

    // Ideally the following would be simply public_storage::read_historical, but we can't implement that yet.
    let mut raw_fields = [0; 3];
    for i in 0..3 {
      raw_fields[i] = public_storage_historical_read(
        context,
        derived_slot + i as Field,
        self.other_contract_address
      );
    }

    let scheduled_value: ScheduledValueChange<T, DELAY> = ScheduledValueChange::deserialize(raw_fields);
    let historical_block_number = context.historical_header.global_variables.block_number as u32;

    (scheduled_value, historical_block_number)
  }

  fn get_derived_storage_slot(self) -> Field {
    // Since we're actually storing three values (a ScheduledValueChange struct), we hash the storage slot to get a
    // unique location in which we can safely store as much data as we need. This could be removed if we informed
    // the slot allocator of how much space we need so that proper padding could be added.
    // See https://github.com/AztecProtocol/aztec-packages/issues/5492
    pedersen_hash([self.storage_slot, 0], 0)
  }
}
