use dep::protocol_types::{hash::pedersen_hash, traits::FromField, address::AztecAddress};

use crate::context::{PrivateContext, Context};
use crate::history::public_storage::public_storage_historical_read;
use crate::public_storage;
use crate::state_vars::{
    storage::Storage,
    shared_mutable::{scheduled_delay_change::ScheduledDelayChange, scheduled_value_change::ScheduledValueChange}
};

struct SharedMutablePrivateGetter<T, INITIAL_DELAY> {
    context: PrivateContext,
    // The contract address of the contract we want to read from
    other_contract_address: AztecAddress,
    // The storage slot where the SharedMutable is stored on the other contract
    storage_slot: Field,
    // The _dummy variable forces INITIAL_DELAY to be interpreted as a numberic value. This is a workaround to
    // https://github.com/noir-lang/noir/issues/4633. Remove once resolved.
    _dummy: [Field; INITIAL_DELAY],
}

// We have this as a view-only interface to reading Shared Mutables in other contracts.
// Currently the Shared Mutable does not support this. We can adapt SharedMutable at a later date
impl<T, INITIAL_DELAY> SharedMutablePrivateGetter<T, INITIAL_DELAY> {
    pub fn new(
        context: PrivateContext,
        other_contract_address: AztecAddress,
        storage_slot: Field
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        assert(other_contract_address.to_field() != 0, "Other contract address cannot be 0");
        Self { context, other_contract_address, storage_slot, _dummy: [0; INITIAL_DELAY] }
    }

    pub fn get_current_value_in_private(self) -> T where T: FromField {
        let mut context = self.context;

        let (value_change, delay_change, historical_block_number) = self.historical_read_from_public_storage(context);
        let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
        let block_horizon = value_change.get_block_horizon(historical_block_number, effective_minimum_delay);

        context.set_tx_max_block_number(block_horizon);
        value_change.get_current_at(historical_block_number)
    }

    fn historical_read_from_public_storage(
        self,
        context: PrivateContext
    ) -> (ScheduledValueChange<T>, ScheduledDelayChange<INITIAL_DELAY>, u32) where T: FromField {
        let value_change_slot = self.get_value_change_storage_slot();
        let mut raw_value_change_fields = [0; 3];
        for i in 0..3 {
            raw_value_change_fields[i] = public_storage_historical_read(
                    context,
                    value_change_slot + i as Field,
                    self.other_contract_address
                );
        }

        let delay_change_slot = self.get_delay_change_storage_slot();
        let raw_delay_change_fields = [public_storage_historical_read(context, delay_change_slot, context.this_address())];

        let value_change = ScheduledValueChange::deserialize(raw_value_change_fields);
        let delay_change = ScheduledDelayChange::deserialize(raw_delay_change_fields);

        let historical_block_number = context.historical_header.global_variables.block_number as u32;

        (value_change, delay_change, historical_block_number)
    }

    fn get_value_change_storage_slot(self) -> Field {
        pedersen_hash([self.storage_slot, 0], 0)
    }

    fn get_delay_change_storage_slot(self) -> Field {
        pedersen_hash([self.storage_slot, 1], 0)
    }
}
