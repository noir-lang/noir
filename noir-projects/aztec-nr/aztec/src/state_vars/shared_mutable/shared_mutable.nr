use dep::protocol_types::{hash::pedersen_hash, traits::FromField};

use crate::context::{PrivateContext, PublicContext};
use crate::public_storage;
use crate::state_vars::{
    storage::Storage,
    shared_mutable::{scheduled_value_change::ScheduledValueChange, scheduled_delay_change::ScheduledDelayChange}
};

mod test;

struct SharedMutable<T, INITIAL_DELAY, Context> {
    context: Context,
    storage_slot: Field,
}

// This will make the Aztec macros require that T implements the Serialize<N> trait, and allocate N storage slots to
// this state variable. This is incorrect, since what we actually store is:
//  - a ScheduledValueChange<T>, which requires 1 + 2 * M storage slots, where M is the serialization length of T
//  - a ScheduledDelayChange, which requires another storage slot
//
// TODO https://github.com/AztecProtocol/aztec-packages/issues/5736: change the storage allocation scheme so that we 
// can actually use it here
impl<T, INITIAL_DELAY, Context> Storage<T> for SharedMutable<T, INITIAL_DELAY, Context> {}

// SharedMutable<T> stores a value of type T that is:
//  - publicly known (i.e. unencrypted)
//  - mutable in public
//  - readable in private with no contention (i.e. multiple parties can all read the same value without blocking one
//    another nor needing to coordinate)
// This is famously a hard problem to solve. SharedMutable makes it work by introducing a delay to public mutation:
// the value is not changed immediately but rather a value change is scheduled to happen in the future after some delay
// measured in blocks. Reads in private are only valid as long as they are included in a block not too far into the 
// future, so that they can guarantee the value will not have possibly changed by then (because of the delay).
// The delay for changing a value is initially equal to INITIAL_DELAY, but can be changed by calling 
// `schedule_delay_change`.
impl<T, INITIAL_DELAY, Context> SharedMutable<T, INITIAL_DELAY, Context> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { context, storage_slot }
    }

    // Since we can't rely on the native storage allocation scheme, we hash the storage slot to get a unique location in
    // which we can safely store as much data as we need. 
    // See https://github.com/AztecProtocol/aztec-packages/issues/5492 and 
    // https://github.com/AztecProtocol/aztec-packages/issues/5736
    fn get_value_change_storage_slot(self) -> Field {
        pedersen_hash([self.storage_slot, 0], 0)
    }

    fn get_delay_change_storage_slot(self) -> Field {
        pedersen_hash([self.storage_slot, 1], 0)
    }
}

impl<T, INITIAL_DELAY> SharedMutable<T, INITIAL_DELAY, &mut PublicContext> {
    pub fn schedule_value_change(self, new_value: T) {
        let mut value_change = self.read_value_change();
        let delay_change = self.read_delay_change();

        let block_number = self.context.block_number() as u32;
        let current_delay = delay_change.get_current(block_number);

        // TODO: make this configurable
        // https://github.com/AztecProtocol/aztec-packages/issues/5501
        let block_of_change = block_number + current_delay;
        value_change.schedule_change(new_value, block_number, current_delay, block_of_change);

        self.write_value_change(value_change);
    }

    pub fn schedule_delay_change(self, new_delay: u32) {
        let mut delay_change = self.read_delay_change();

        let block_number = self.context.block_number() as u32;

        delay_change.schedule_change(new_delay, block_number);

        self.write_delay_change(delay_change);
    }

    pub fn get_current_value_in_public(self) -> T {
        let block_number = self.context.block_number() as u32;
        self.read_value_change().get_current_at(block_number)
    }

    pub fn get_current_delay_in_public(self) -> u32 {
        let block_number = self.context.block_number() as u32;
        self.read_delay_change().get_current(block_number)
    }

    pub fn get_scheduled_value_in_public(self) -> (T, u32) {
        self.read_value_change().get_scheduled()
    }

    pub fn get_scheduled_delay_in_public(self) -> (u32, u32) {
        self.read_delay_change().get_scheduled()
    }

    fn read_value_change(self) -> ScheduledValueChange<T> {
        public_storage::read(self.get_value_change_storage_slot())
    }

    fn read_delay_change(self) -> ScheduledDelayChange<INITIAL_DELAY> {
        public_storage::read(self.get_delay_change_storage_slot())
    }

    fn write_value_change(self, value_change: ScheduledValueChange<T>) {
        public_storage::write(self.get_value_change_storage_slot(), value_change);
    }

    fn write_delay_change(self, delay_change: ScheduledDelayChange<INITIAL_DELAY>) {
        public_storage::write(self.get_delay_change_storage_slot(), delay_change);
    }
}

impl<T, INITIAL_DELAY> SharedMutable<T, INITIAL_DELAY, &mut PrivateContext> {
    pub fn get_current_value_in_private(self) -> T where T: FromField {
        // When reading the current value in private we construct a historical state proof for the public value.
        // However, since this value might change, we must constrain the maximum transaction block number as this proof
        // will only be valid for however many blocks we can ensure the value will not change, which will depend on the
        // current delay and any scheduled delay changes.

        let (value_change, delay_change, historical_block_number) = self.historical_read_from_public_storage(*self.context);

        // We use the effective minimum delay as opposed to the current delay at the historical block as this one also
        // takes into consideration any scheduled delay changes. 
        // For example, consider a scenario in which at block 200 the current delay was 50. We may naively think that
        // the earliest we could change the value would be at block 251 by scheduling immediately after the historical
        // block, i.e. at block 201. But if there was a delay change scheduled for block 210 to reduce the delay to 20 
        // blocks, then if a value change was scheduled at block 210 it would go into effect at block 230, which is 
        // earlier than what we'd expect if we only considered the current delay.
        let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
        let block_horizon = value_change.get_block_horizon(historical_block_number, effective_minimum_delay);

        // We prevent this transaction from being included in any block after the block horizon, ensuring that the 
        // historical public value matches the current one, since it can only change after the horizon.
        self.context.set_tx_max_block_number(block_horizon);
        value_change.get_current_at(historical_block_number)
    }

    fn historical_read_from_public_storage(
        self,
        context: PrivateContext
    ) -> (ScheduledValueChange<T>, ScheduledDelayChange<INITIAL_DELAY>, u32) where T: FromField {
        let header = context.get_header();
        // Ideally the following would be simply public_storage::read_historical, but we can't implement that yet.
        let value_change_slot = self.get_value_change_storage_slot();
        let mut raw_value_change_fields = [0; 3];
        for i in 0..3 {
            raw_value_change_fields[i] = header.public_storage_historical_read(
                value_change_slot + i as Field,
                context.this_address()
            );
        }

        // Ideally the following would be simply public_storage::read_historical, but we can't implement that yet.
        let delay_change_slot = self.get_delay_change_storage_slot();
        let raw_delay_change_fields = [header.public_storage_historical_read(delay_change_slot, context.this_address())];

        let value_change = ScheduledValueChange::deserialize(raw_value_change_fields);
        let delay_change = ScheduledDelayChange::deserialize(raw_delay_change_fields);

        let historical_block_number = context.historical_header.global_variables.block_number as u32;

        (value_change, delay_change, historical_block_number)
    }
}
