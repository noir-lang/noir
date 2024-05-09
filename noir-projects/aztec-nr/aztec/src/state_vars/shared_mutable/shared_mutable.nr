use dep::protocol_types::{hash::pedersen_hash, traits::FromField};

use crate::context::{PrivateContext, PublicContext, Context};
use crate::history::public_storage::public_storage_historical_read;
use crate::public_storage;
use crate::state_vars::{
    storage::Storage,
    shared_mutable::{scheduled_value_change::ScheduledValueChange, scheduled_delay_change::ScheduledDelayChange}
};

struct SharedMutable<T, INITIAL_DELAY> {
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
impl<T, INITIAL_DELAY> Storage<T> for SharedMutable<T, INITIAL_DELAY> {}

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
impl<T, INITIAL_DELAY> SharedMutable<T, INITIAL_DELAY> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { context, storage_slot }
    }

    pub fn schedule_value_change(self, new_value: T) {
        let context = self.context.public.unwrap();
        let mut value_change = self.read_value_change();
        let delay_change = self.read_delay_change();

        let block_number = context.block_number() as u32;
        let current_delay = delay_change.get_current(block_number);

        // TODO: make this configurable
        // https://github.com/AztecProtocol/aztec-packages/issues/5501
        let block_of_change = block_number + current_delay;
        value_change.schedule_change(new_value, block_number, current_delay, block_of_change);

        self.write_value_change(value_change);
    }

    pub fn schedule_delay_change(self, new_delay: u32) {
        let context = self.context.public.unwrap();
        let mut delay_change = self.read_delay_change();

        let block_number = context.block_number() as u32;

        delay_change.schedule_change(new_delay, block_number);

        self.write_delay_change(delay_change);
    }

    pub fn get_current_value_in_public(self) -> T {
        let block_number = self.context.public.unwrap().block_number() as u32;
        self.read_value_change().get_current_at(block_number)
    }

    pub fn get_current_delay_in_public(self) -> u32 {
        let block_number = self.context.public.unwrap().block_number() as u32;
        self.read_delay_change().get_current(block_number)
    }

    pub fn get_scheduled_value_in_public(self) -> (T, u32) {
        self.read_value_change().get_scheduled()
    }

    pub fn get_scheduled_delay_in_public(self) -> (u32, u32) {
        self.read_delay_change().get_scheduled()
    }

    pub fn get_current_value_in_private(self) -> T where T: FromField {
        let mut context = self.context.private.unwrap();

        // When reading the current value in private we construct a historical state proof for the public value.
        // However, since this value might change, we must constrain the maximum transaction block number as this proof
        // will only be valid for however many blocks we can ensure the value will not change, which will depend on the
        // current delay and any scheduled delay changes.

        let (value_change, delay_change, historical_block_number) = self.historical_read_from_public_storage(*context);

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
        context.set_tx_max_block_number(block_horizon);
        value_change.get_current_at(historical_block_number)
    }

    fn historical_read_from_public_storage(
        self,
        context: PrivateContext
    ) -> (ScheduledValueChange<T>, ScheduledDelayChange<INITIAL_DELAY>, u32) where T: FromField {
        // Ideally the following would be simply public_storage::read_historical, but we can't implement that yet.
        let value_change_slot = self.get_value_change_storage_slot();
        let mut raw_value_change_fields = [0; 3];
        for i in 0..3 {
            raw_value_change_fields[i] = public_storage_historical_read(
                    context,
                    value_change_slot + i as Field,
                    context.this_address()
                );
        }

        // Ideally the following would be simply public_storage::read_historical, but we can't implement that yet.
        let delay_change_slot = self.get_delay_change_storage_slot();
        let raw_delay_change_fields = [public_storage_historical_read(context, delay_change_slot, context.this_address())];

        let value_change = ScheduledValueChange::deserialize(raw_value_change_fields);
        let delay_change = ScheduledDelayChange::deserialize(raw_delay_change_fields);

        let historical_block_number = context.historical_header.global_variables.block_number as u32;

        (value_change, delay_change, historical_block_number)
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

mod test {
    use dep::std::{merkle::compute_merkle_root, test::OracleMock};

    use crate::{
        context::{PublicContext, PrivateContext, Context},
        state_vars::shared_mutable::{
        shared_mutable::SharedMutable, scheduled_value_change::ScheduledValueChange,
        scheduled_delay_change::ScheduledDelayChange
    },
        oracle::get_public_data_witness::PublicDataWitness
    };

    use dep::protocol_types::{
        constants::{GENERATOR_INDEX__PUBLIC_LEAF_INDEX, PUBLIC_DATA_TREE_HEIGHT}, hash::pedersen_hash,
        address::AztecAddress, public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage
    };

    global pre_value = 13;
    global post_value = 42;

    global new_value = 57;

    global pre_delay = 20;
    global post_delay = 15;

    global TEST_INITIAL_DELAY = 3;

    fn setup(private: bool) -> (SharedMutable<Field, TEST_INITIAL_DELAY>, Field) {
        let block_number = 40;
        let context = create_context(block_number, private);

        let storage_slot = 57;
        let state_var: SharedMutable<Field, TEST_INITIAL_DELAY> = SharedMutable::new(context, storage_slot);

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

    fn mock_value_change_read(
        state_var: SharedMutable<Field, TEST_INITIAL_DELAY>,
        pre: Field,
        post: Field,
        block_of_change: Field
    ) {
        let value_change_slot = state_var.get_value_change_storage_slot();
        let fields = ScheduledValueChange::new(pre, post, block_of_change as u32).serialize();

        let _ = OracleMock::mock("storageRead").with_params((value_change_slot, 3)).returns(fields).times(1);
    }

    fn mock_delay_change_read(
        state_var: SharedMutable<Field, TEST_INITIAL_DELAY>,
        pre: Field,
        post: Field,
        block_of_change: Field
    ) {
        let delay_change_slot = state_var.get_delay_change_storage_slot();
        let delay_change: ScheduledDelayChange<TEST_INITIAL_DELAY> = ScheduledDelayChange::new(
            Option::some(pre as u32),
            Option::some(post as u32),
            block_of_change as u32
        );
        let fields = delay_change.serialize();

        let _ = OracleMock::mock("storageRead").with_params((delay_change_slot, 1)).returns(fields).times(1);
    }

    fn mock_delay_change_read_uninitialized(state_var: SharedMutable<Field, TEST_INITIAL_DELAY>) {
        let delay_change_slot = state_var.get_delay_change_storage_slot();
        let _ = OracleMock::mock("storageRead").with_params((delay_change_slot, 1)).returns([0]).times(1);
    }

    // Useful since change and delay values are always the global pre/post ones, so we typically only care about their
    // block of change.
    fn mock_value_and_delay_read(
        state_var: SharedMutable<Field, TEST_INITIAL_DELAY>,
        value_block_of_change: Field,
        delay_block_of_change: Field
    ) {
        mock_value_change_read(state_var, pre_value, post_value, value_block_of_change);
        mock_delay_change_read(state_var, pre_delay, post_delay, delay_block_of_change);
    }

    fn mock_value_change_write() -> OracleMock {
        OracleMock::mock("storageWrite").returns([0; 3])
    }

    fn mock_delay_change_write() -> OracleMock {
        OracleMock::mock("storageWrite").returns([0; 1])
    }

    fn assert_value_change_write(
        state_var: SharedMutable<Field, TEST_INITIAL_DELAY>,
        mock: OracleMock,
        pre: Field,
        post: Field,
        block_of_change: Field
    ) {
        let fields = ScheduledValueChange::new(pre, post, block_of_change as u32).serialize();
        assert_eq(mock.get_last_params(), (state_var.get_value_change_storage_slot(), fields));
    }

    fn assert_delay_change_write(
        state_var: SharedMutable<Field, TEST_INITIAL_DELAY>,
        mock: OracleMock,
        pre: Field,
        post: Field,
        block_of_change: Field
    ) {
        let delay_change: ScheduledDelayChange<TEST_INITIAL_DELAY> = ScheduledDelayChange::new(
            Option::some(pre as u32),
            Option::some(post as u32),
            block_of_change as u32
        );

        let fields = delay_change.serialize();
        assert_eq(mock.get_last_params(), (state_var.get_delay_change_storage_slot(), fields));
    }

    #[test]
    fn test_get_current_value_in_public() {
        let (state_var, block_number) = setup(false);

        // Change in the future, current value is pre
        mock_value_change_read(state_var, pre_value, post_value, block_number + 1);
        assert_eq(state_var.get_current_value_in_public(), pre_value);

        // Change in the current block, current value is post
        mock_value_change_read(state_var, pre_value, post_value, block_number);
        assert_eq(state_var.get_current_value_in_public(), post_value);

        // Change in the past, current value is post
        mock_value_change_read(state_var, pre_value, post_value, block_number - 1);
        assert_eq(state_var.get_current_value_in_public(), post_value);
    }

    #[test]
    fn test_get_scheduled_value_in_public() {
        let (state_var, block_number) = setup(false);

        // Change in the future, scheduled is post (always is)
        mock_value_change_read(state_var, pre_value, post_value, block_number + 1);
        assert_eq(state_var.get_scheduled_value_in_public(), (post_value, (block_number + 1) as u32));

        // Change in the current block, scheduled is post (always is)
        mock_value_change_read(state_var, pre_value, post_value, block_number);
        assert_eq(state_var.get_scheduled_value_in_public(), (post_value, block_number as u32));

        // Change in the past, scheduled is post (always is)
        mock_value_change_read(state_var, pre_value, post_value, block_number - 1);
        assert_eq(state_var.get_scheduled_value_in_public(), (post_value, (block_number - 1) as u32));
    }

    #[test]
    fn test_get_current_delay_in_public() {
        let (state_var, block_number) = setup(false);

        // Uninitialized
        mock_delay_change_read_uninitialized(state_var);
        assert_eq(state_var.get_current_delay_in_public(), TEST_INITIAL_DELAY as u32);

        // Change in the future, current value is pre
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
        assert_eq(state_var.get_current_delay_in_public(), pre_delay as u32);

        // Change in the current block, current value is post
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number);
        assert_eq(state_var.get_current_delay_in_public(), post_delay as u32);

        // Change in the past, current value is post
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
        assert_eq(state_var.get_current_delay_in_public(), post_delay as u32);
    }

    #[test]
    fn test_get_scheduled_delay_in_public_before_change() {
        let (state_var, block_number) = setup(false);

        // Uninitialized
        mock_delay_change_read_uninitialized(state_var);
        assert_eq(state_var.get_scheduled_delay_in_public(), (TEST_INITIAL_DELAY as u32, 0));

        // Change in the future, scheduled is post (always is)
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
        assert_eq(state_var.get_scheduled_delay_in_public(), (post_delay as u32, (block_number + 1) as u32));

        // Change in the current block, scheduled is post (always is)
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number);
        assert_eq(state_var.get_scheduled_delay_in_public(), (post_delay as u32, block_number as u32));

        // Change in the past, scheduled is post (always is)
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
        assert_eq(state_var.get_scheduled_delay_in_public(), (post_delay as u32, (block_number - 1) as u32));
    }

    #[test]
    fn test_schedule_value_change_no_delay() {
        let (state_var, block_number) = setup(false);

        // Last value change was in the past
        mock_value_change_read(state_var, pre_value, post_value, 0);

        // Current delay is 0
        mock_delay_change_read(state_var, 0, 0, block_number);

        let write_mock = mock_value_change_write();

        state_var.schedule_value_change(new_value);

        // The new value has a block of change equal to the current block, i.e. it is the current value
        assert_value_change_write(state_var, write_mock, post_value, new_value, block_number);
    }

    #[test]
    fn test_schedule_value_change_before_change_no_scheduled_delay() {
        let (state_var, block_number) = setup(false);

        // Value change in the future, delay change in the past
        mock_value_and_delay_read(state_var, block_number + 1, block_number - 1);
        let write_mock = mock_value_change_write();

        state_var.schedule_value_change(new_value);

        // The new scheduled value change replaces the old one, post delay (current) is used
        assert_value_change_write(
            state_var,
            write_mock,
            pre_value,
            new_value,
            block_number + post_delay
        );
    }

    #[test]
    fn test_schedule_value_change_before_change_scheduled_delay() {
        let (state_var, block_number) = setup(false);

        // Value change in the future, delay change in the future
        mock_value_and_delay_read(state_var, block_number + 1, block_number + 1);

        let write_mock = mock_value_change_write();

        state_var.schedule_value_change(new_value);

        // The new scheduled value change replaces the old one, pre delay (current, not scheduled) is used
        assert_value_change_write(
            state_var,
            write_mock,
            pre_value,
            new_value,
            block_number + pre_delay
        );
    }

    #[test]
    fn test_schedule_value_change_after_change_no_scheduled_delay() {
        let (state_var, block_number) = setup(false);

        // Value change in the past, delay change in the past
        mock_value_and_delay_read(state_var, block_number - 1, block_number - 1);
        let write_mock = mock_value_change_write();

        state_var.schedule_value_change(new_value);

        // The previous post value becomes the pre value, post delay (current) is used
        assert_value_change_write(
            state_var,
            write_mock,
            post_value,
            new_value,
            block_number + post_delay
        );
    }

    #[test]
    fn test_schedule_value_change_after_change_scheduled_delay() {
        let (state_var, block_number) = setup(false);

        // Value change in the past, delay change in the future
        mock_value_and_delay_read(state_var, block_number - 1, block_number + 1);

        let write_mock = mock_value_change_write();

        state_var.schedule_value_change(new_value);

        // The previous post value becomes the pre value, pre delay (current, not scheduled) is used
        assert_value_change_write(
            state_var,
            write_mock,
            post_value,
            new_value,
            block_number + pre_delay
        );
    }

    #[test]
    fn test_schedule_delay_increase_before_change() {
        let (state_var, block_number) = setup(false);

        // Delay change in future, current delay is pre
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
        let write_mock = mock_delay_change_write();

        let new_delay = pre_delay + 1;
        state_var.schedule_delay_change(new_delay as u32);

        // The previous scheduled change is lost, change is immediate (due to increase)
        assert_delay_change_write(state_var, write_mock, pre_delay, new_delay, block_number);
    }

    #[test]
    fn test_schedule_delay_reduction_before_change() {
        let (state_var, block_number) = setup(false);

        // Delay change in future, current delay is pre
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
        let write_mock = mock_delay_change_write();

        let new_delay = pre_delay - 1;
        state_var.schedule_delay_change(new_delay as u32);

        // The previous scheduled change is lost, change delay equals difference (due to reduction)
        assert_delay_change_write(
            state_var,
            write_mock,
            pre_delay,
            new_delay,
            block_number + pre_delay - new_delay
        );
    }

    #[test]
    fn test_schedule_delay_increase_after_change() {
        let (state_var, block_number) = setup(false);

        // Delay change in the past, current delay is post
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
        let write_mock = mock_delay_change_write();

        let new_delay = post_delay + 1;
        state_var.schedule_delay_change(new_delay as u32);

        // The current value becomes pre, change is immediate (due to increase)
        assert_delay_change_write(state_var, write_mock, post_delay, new_delay, block_number);
    }

    #[test]
    fn test_schedule_delay_reduction_after_change() {
        let (state_var, block_number) = setup(false);

        // Delay change in the past, current delay is post
        mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
        let write_mock = mock_delay_change_write();

        let new_delay = post_delay - 1;
        state_var.schedule_delay_change(new_delay as u32);

        // The current value becomes pre, change delay equals difference (due to reduction)
        assert_delay_change_write(
            state_var,
            write_mock,
            post_delay,
            new_delay,
            block_number + post_delay - new_delay
        );
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
