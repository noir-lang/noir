use dep::std::{merkle::compute_merkle_root, test::OracleMock};

use crate::{
    context::{PublicContext, PrivateContext},
    state_vars::shared_mutable::{
    shared_mutable::SharedMutable, scheduled_value_change::ScheduledValueChange,
    scheduled_delay_change::ScheduledDelayChange
},
    test::helpers::context_builder::ContextBuilder, oracle::get_public_data_witness::PublicDataWitness
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

fn setup() -> (SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext>, Field) {
    let block_number = 40;
    let mut context = ContextBuilder::new().block_number(block_number).public();

    let storage_slot = 57;
    let state_var = SharedMutable::new(&mut context, storage_slot);

    (state_var, block_number)
}

fn mock_value_change_read(
    state_var: SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext>,
    pre: Field,
    post: Field,
    block_of_change: Field
) {
    let value_change_slot = state_var.get_value_change_storage_slot();
    let fields = ScheduledValueChange::new(pre, post, block_of_change as u32).serialize();

    let _ = OracleMock::mock("storageRead").with_params((value_change_slot, 3)).returns(fields).times(1);
}

fn mock_delay_change_read(
    state_var: SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext>,
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

fn mock_delay_change_read_uninitialized(state_var: SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext>) {
    let delay_change_slot = state_var.get_delay_change_storage_slot();
    let _ = OracleMock::mock("storageRead").with_params((delay_change_slot, 1)).returns([0]).times(1);
}

// Useful since change and delay values are always the global pre/post ones, so we typically only care about their
// block of change.
fn mock_value_and_delay_read(
    state_var: SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext>,
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
    state_var: SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext>,
    mock: OracleMock,
    pre: Field,
    post: Field,
    block_of_change: Field
) {
    let fields = ScheduledValueChange::new(pre, post, block_of_change as u32).serialize();
    assert_eq(mock.get_last_params(), (state_var.get_value_change_storage_slot(), fields));
}

fn assert_delay_change_write(
    state_var: SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext>,
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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
    let (state_var, block_number) = setup();

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
