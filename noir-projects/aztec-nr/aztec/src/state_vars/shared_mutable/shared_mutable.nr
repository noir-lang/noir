use dep::protocol_types::{hash::pedersen_hash, traits::FromField};

use crate::context::{PrivateContext, PublicContext, Context};
use crate::history::public_storage::public_storage_historical_read;
use crate::public_storage;
use crate::state_vars::{storage::Storage, shared_mutable::scheduled_value_change::ScheduledValueChange};

struct SharedMutable<T, DELAY> {
    context: Context,
    storage_slot: Field,
}

impl<T, DELAY> Storage<T> for SharedMutable<T, DELAY> {}

// SharedMutable<T> stores a value of type T that is:
//  - publicly known (i.e. unencrypted)
//  - mutable in public
//  - readable in private with no contention (i.e. multiple parties can all read the same value without blocking one
//    another nor needing to coordinate)
// This is famously a hard problem to solve. SharedMutable makes it work by introducing a delay to public mutation:
// the value is not changed immediately but rather a value change is scheduled to happen in the future after some delay
// measured in blocks. Reads in private are only valid as long as they are included in a block not too far into the 
// future, so that they can guarantee the value will not have possibly changed by then (because of the delay).
impl<T, DELAY> SharedMutable<T, DELAY> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { context, storage_slot }
    }

    pub fn schedule_value_change(self, new_value: T) {
        let context = self.context.public.unwrap();
        let mut scheduled_value_change: ScheduledValueChange<T, DELAY> = public_storage::read(self.get_derived_storage_slot());

        scheduled_value_change.schedule_change(new_value, context.block_number() as u32);

        public_storage::write(self.get_derived_storage_slot(), scheduled_value_change);
    }

    pub fn get_current_value_in_public(self) -> T {
        let scheduled_value_change: ScheduledValueChange<T, DELAY> = public_storage::read(self.get_derived_storage_slot());

        let block_number = self.context.public.unwrap().block_number() as u32;
        scheduled_value_change.get_current_at(block_number)
    }

    pub fn get_scheduled_value_in_public(self) -> (T, u32) {
        let scheduled_value_change: ScheduledValueChange<T, DELAY> = public_storage::read(self.get_derived_storage_slot());
        scheduled_value_change.get_scheduled()
    }

    pub fn get_current_value_in_private(self) -> T where T: FromField {
        let mut context = self.context.private.unwrap();

        let (scheduled_value_change, historical_block_number) = self.historical_read_from_public_storage(*context);
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
                    context.this_address()
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

mod test {
    use dep::std::{merkle::compute_merkle_root, test::OracleMock};

    use crate::{
        context::{PublicContext, PrivateContext, Context},
        state_vars::shared_mutable::shared_mutable::SharedMutable,
        oracle::get_public_data_witness::PublicDataWitness
    };

    use dep::protocol_types::{
        constants::{GENERATOR_INDEX__PUBLIC_LEAF_INDEX, PUBLIC_DATA_TREE_HEIGHT}, hash::pedersen_hash,
        address::AztecAddress, public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage
    };

    fn setup(private: bool) -> (SharedMutable<Field, TEST_DELAY>, Field) {
        let block_number = 40;
        let context = create_context(block_number, private);

        let storage_slot = 57;
        let state_var: SharedMutable<Field, TEST_DELAY> = SharedMutable::new(context, storage_slot);

        (state_var, block_number)
    }

    fn create_context(block_number: Field, private: bool) -> Context {
        if private {
            let mut private_context = PrivateContext::empty();
            private_context.historical_header.global_variables.block_number = block_number;
            Context::private(&mut private_context)
        } else {
            let mut public_context = PublicContext::empty();
            public_context.inputs.public_global_variables.block_number = block_number;
            Context::public(&mut public_context)
        }
    }

    global TEST_DELAY = 20;

    global pre = 13;
    global post = 42;

    #[test]
    fn test_get_current_value_in_public_before_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the future, current value is pre
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number + 1]);
        assert_eq(state_var.get_current_value_in_public(), pre);
    }

    #[test]
    fn test_get_current_value_in_public_at_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the current block, current value is post
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number]);
        assert_eq(state_var.get_current_value_in_public(), post);
    }

    #[test]
    fn test_get_current_value_in_public_after_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the past, current value is post
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number - 1]);
        assert_eq(state_var.get_current_value_in_public(), post);
    }

    #[test]
    fn test_get_scheduled_value_in_public_before_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the future, scheduled is post (always is)
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number + 1]);
        assert_eq(state_var.get_scheduled_value_in_public(), (post, (block_number + 1) as u32));
    }

    #[test]
    fn test_get_scheduled_value_in_public_at_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the current block, scheduled is post (always is)
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number]);
        assert_eq(state_var.get_scheduled_value_in_public(), (post, block_number as u32));
    }

    #[test]
    fn test_get_scheduled_value_in_public_after_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the past, scheduled is post (always is)
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number - 1]);
        assert_eq(state_var.get_scheduled_value_in_public(), (post, (block_number - 1) as u32));
    }

    #[test]
    fn test_schedule_value_change_before_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the future
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number + 1]);

        let write_mock = OracleMock::mock("storageWrite").returns([0; 3]); // The oracle return value is actually unused

        let new_value = 42;
        state_var.schedule_value_change(new_value);

        // The new scheduled change replaces the old one
        assert_eq(write_mock.get_last_params(), (slot, [pre, new_value, block_number + TEST_DELAY]));
    }

    #[test]
    fn test_schedule_value_change_at_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the current block
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number]);

        let write_mock = OracleMock::mock("storageWrite").returns([0; 3]); // The oracle return value is actually unused

        let new_value = 42;
        state_var.schedule_value_change(new_value);

        // The previous 'post' value is the current one and becomes the 'pre' value
        assert_eq(write_mock.get_last_params(), (slot, [post, new_value, block_number + TEST_DELAY]));
    }

    #[test]
    fn test_schedule_value_change_after_change() {
        let (state_var, block_number) = setup(false);

        let slot = state_var.get_derived_storage_slot();

        // Change in the past
        OracleMock::mock("storageRead").with_params((slot, 3)).returns([pre, post, block_number - 1]);

        let write_mock = OracleMock::mock("storageWrite").returns([0; 3]); // The oracle return value is actually unused

        let new_value = 42;
        state_var.schedule_value_change(new_value);

        // The previous 'post' value is the current one and becomes the 'pre' value
        assert_eq(write_mock.get_last_params(), (slot, [post, new_value, block_number + TEST_DELAY]));
    }

    #[test]
    fn test_get_current_value_in_private_before_change() {
        // Here we'd want to test that the private getter returns the correct value and sets max_block_number in the
        // context to the expected block horizon, in all the possible scenarios (long before change, before near change,
        // after change).
        // However, this requires mocking the getPublicDataTreeWitness oracle so that we can convince the circuit that
        // it got a valid historical proof. Because we can set the tree root to whatever we want in the context, this is
        // trivial for a single historical value (we add a leaf and compute the root with any random path), but is quite
        // hard if we're reading more than one value for the same root (as SharedMutable does): we essentially need to
        // create an actual indexed tree and compute the correct path for each of the inserted values.
        // TODO: implement an actual tree and use it here https://github.com/AztecProtocol/aztec-packages/issues/5494
    }
}
