use crate::{
    context::{PublicContext, PrivateContext},
    state_vars::shared_mutable::{
    shared_mutable::SharedMutable, scheduled_value_change::ScheduledValueChange,
    scheduled_delay_change::ScheduledDelayChange
},
    test::helpers::test_environment::TestEnvironment
};

use dep::protocol_types::address::AztecAddress;
use dep::std::{test::OracleMock, unsafe::zeroed};

global new_value = 17;

global new_delay = 20;

global storage_slot = 47;

global TEST_INITIAL_DELAY: u32 = 32;

fn setup() -> TestEnvironment {
    TestEnvironment::new()
}

fn in_public(env: TestEnvironment) -> SharedMutable<Field, TEST_INITIAL_DELAY, &mut PublicContext> {
    SharedMutable::new(&mut env.public(), storage_slot)
}

fn in_private(
    env: &mut TestEnvironment,
    historical_block_number: u32
) -> SharedMutable<Field, TEST_INITIAL_DELAY, &mut PrivateContext> {
    SharedMutable::new(&mut env.private_at(historical_block_number), storage_slot)
}

#[test]
fn test_get_current_value_in_public_initial() {
    let env = setup();
    let state_var = in_public(env);

    assert_eq(state_var.get_current_value_in_public(), zeroed());
}

#[test]
fn test_get_scheduled_value_in_public() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_value_change(new_value);

    let (scheduled, block_of_change) = state_var.get_scheduled_value_in_public();
    assert_eq(scheduled, new_value);
    assert_eq(block_of_change, env.block_number() + TEST_INITIAL_DELAY);
}

#[test]
fn test_get_current_value_in_public_before_scheduled_change() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_value_change(new_value);

    let (_, block_of_change) = state_var.get_scheduled_value_in_public();

    let original_value = zeroed();

    // The current value has not changed
    assert_eq(state_var.get_current_value_in_public(), original_value);

    // The current value still does not change right before the block of change
    env.advance_block_to(block_of_change - 1);
    assert_eq(state_var.get_current_value_in_public(), original_value);
}

#[test]
fn test_get_current_value_in_public_at_scheduled_change() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_value_change(new_value);

    let (_, block_of_change) = state_var.get_scheduled_value_in_public();

    env.advance_block_to(block_of_change);
    assert_eq(state_var.get_current_value_in_public(), new_value);
}

#[test]
fn test_get_current_value_in_public_after_scheduled_change() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_value_change(new_value);

    let (_, block_of_change) = state_var.get_scheduled_value_in_public();

    env.advance_block_to(block_of_change + 10);
    assert_eq(state_var.get_current_value_in_public(), new_value);
}

#[test]
fn test_get_current_delay_in_public_initial() {
    let env = setup();
    let state_var = in_public(env);

    assert_eq(state_var.get_current_delay_in_public(), TEST_INITIAL_DELAY);
}

#[test]
fn test_get_scheduled_delay_in_public() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_delay_change(new_delay);

    let (scheduled, block_of_change) = state_var.get_scheduled_delay_in_public();
    assert_eq(scheduled, new_delay);
    // The new delay is smaller, therefore we need to wait for the difference between current and new
    assert_eq(block_of_change, env.block_number() + TEST_INITIAL_DELAY - new_delay);
}

#[test]
fn test_get_current_delay_in_public_before_scheduled_change() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_delay_change(new_delay);

    let (_, block_of_change) = state_var.get_scheduled_delay_in_public();

    let original_delay = TEST_INITIAL_DELAY;

    // The current delay has not changed
    assert_eq(state_var.get_current_delay_in_public(), original_delay);

    // The current delay still does not change right before the block of change
    env.advance_block_to(block_of_change - 1);
    assert_eq(state_var.get_current_delay_in_public(), original_delay);
}

#[test]
fn test_get_current_delay_in_public_at_scheduled_change() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_delay_change(new_delay);

    let (_, block_of_change) = state_var.get_scheduled_delay_in_public();

    env.advance_block_to(block_of_change);
    assert_eq(state_var.get_current_delay_in_public(), new_delay);
}

#[test]
fn test_get_current_delay_in_public_after_scheduled_change() {
    let mut env = setup();
    let state_var = in_public(env);

    state_var.schedule_delay_change(new_delay);

    let (_, block_of_change) = state_var.get_scheduled_delay_in_public();

    env.advance_block_to(block_of_change + 10);
    assert_eq(state_var.get_current_delay_in_public(), new_delay);
}

#[test]
fn test_get_current_value_in_private_initial() {
    let mut env = setup();

    let historical_block_number = env.block_number();
    let state_var = in_private(&mut env, historical_block_number);

    assert_eq(state_var.get_current_value_in_private(), zeroed());
    assert_eq(state_var.context.max_block_number.unwrap(), historical_block_number + TEST_INITIAL_DELAY);
}

#[test]
fn test_get_current_value_in_private_before_change() {
    let mut env = setup();

    let public_state_var = in_public(env);
    public_state_var.schedule_value_change(new_value);

    let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

    let schedule_block_number = env.block_number();

    let private_state_var = in_private(&mut env, schedule_block_number);
    assert_eq(private_state_var.get_current_value_in_private(), 0);
    assert_eq(private_state_var.context.max_block_number.unwrap(), block_of_change - 1);
}

#[test]
fn test_get_current_value_in_private_immediately_before_change() {
    let mut env = setup();

    let public_state_var = in_public(env);
    public_state_var.schedule_value_change(new_value);

    let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

    let private_state_var = in_private(&mut env, block_of_change - 1);

    // Note that this transaction would never be valid since the max block number is the same as the historical block
    // used to built the proof, i.e. in the past.
    assert_eq(private_state_var.get_current_value_in_private(), 0);
    assert_eq(private_state_var.context.max_block_number.unwrap(), block_of_change - 1);
}

#[test]
fn test_get_current_value_in_private_at_change() {
    let mut env = setup();

    let public_state_var = in_public(env);
    public_state_var.schedule_value_change(new_value);

    let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

    let historical_block_number = block_of_change;
    let private_state_var = in_private(&mut env, historical_block_number);
    assert_eq(private_state_var.get_current_value_in_private(), new_value);
    assert_eq(
        private_state_var.context.max_block_number.unwrap(), historical_block_number + TEST_INITIAL_DELAY
    );
}

#[test]
fn test_get_current_value_in_private_after_change() {
    let mut env = setup();

    let public_state_var = in_public(env);
    public_state_var.schedule_value_change(new_value);

    let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

    let historical_block_number = block_of_change + 10;
    let private_state_var = in_private(&mut env, historical_block_number);
    assert_eq(private_state_var.get_current_value_in_private(), new_value);
    assert_eq(
        private_state_var.context.max_block_number.unwrap(), historical_block_number + TEST_INITIAL_DELAY
    );
}

#[test]
fn test_get_current_value_in_private_with_non_initial_delay() {
    let mut env = setup();

    let public_state_var = in_public(env);
    public_state_var.schedule_value_change(new_value);
    public_state_var.schedule_delay_change(new_delay);

    let (_, value_block_of_change) = public_state_var.get_scheduled_value_in_public();
    let (_, delay_block_of_change) = public_state_var.get_scheduled_delay_in_public();

    let historical_block_number = if value_block_of_change > delay_block_of_change {
        value_block_of_change
    } else {
        delay_block_of_change
    };

    let private_state_var = in_private(&mut env, historical_block_number);
    assert_eq(private_state_var.get_current_value_in_private(), new_value);
    assert_eq(private_state_var.context.max_block_number.unwrap(), historical_block_number + new_delay);
}

#[test(should_fail_with="Hint values do not match hash")]
fn test_get_current_value_in_private_bad_value_hints() {
    let mut env = setup();

    let public_state_var = in_public(env);
    public_state_var.schedule_value_change(new_value);

    let schedule_block_number = env.block_number();
    let private_state_var = in_private(&mut env, schedule_block_number);

    let mocked: ScheduledValueChange<Field> = ScheduledValueChange::new(0, new_value + 1, schedule_block_number);
    let _ = OracleMock::mock("storageRead").with_params(
        (
            env.contract_address().to_field(), private_state_var.get_value_change_storage_slot(), schedule_block_number, 3
        )
    ).returns(mocked.serialize()).times(1);

    let _ = private_state_var.get_current_value_in_private();
}

#[test(should_fail_with="Hint values do not match hash")]
fn test_get_current_value_in_private_bad_delay_hints() {
    let mut env = setup();

    let public_state_var = in_public(env);
    public_state_var.schedule_value_change(new_value);

    let schedule_block_number = env.block_number();
    let private_state_var = in_private(&mut env, schedule_block_number);

    let mocked: ScheduledDelayChange<TEST_INITIAL_DELAY> = ScheduledDelayChange::new(Option::none(), Option::some(42), schedule_block_number);
    let _ = OracleMock::mock("storageRead").with_params(
        (
            env.contract_address().to_field(), private_state_var.get_delay_change_storage_slot(), schedule_block_number, 1
        )
    ).returns(mocked.serialize()).times(1);

    let _ = private_state_var.get_current_value_in_private();
}

#[test(should_fail_with="Non-zero value change for zero hash")]
fn test_get_current_value_in_private_bad_zero_hash_value_hints() {
    let mut env = setup();

    let historical_block_number = env.block_number();
    let state_var = in_private(&mut env, historical_block_number);

    let mocked: ScheduledValueChange<Field> = ScheduledValueChange::new(0, new_value, 0);
    let _ = OracleMock::mock("storageRead").with_params(
        (
            env.contract_address().to_field(), state_var.get_value_change_storage_slot(), historical_block_number, 3
        )
    ).returns(mocked.serialize()).times(1);

    let _ = state_var.get_current_value_in_private();
}

#[test(should_fail_with="Non-zero delay change for zero hash")]
fn test_get_current_value_in_private_bad_zero_hash_delay_hints() {
    let mut env = setup();

    let historical_block_number = env.block_number();
    let state_var = in_private(&mut env, historical_block_number);

    let mocked: ScheduledDelayChange<TEST_INITIAL_DELAY> = ScheduledDelayChange::new(Option::none(), Option::some(new_delay), 0);
    let _ = OracleMock::mock("storageRead").with_params(
        (
            env.contract_address().to_field(), state_var.get_delay_change_storage_slot(), historical_block_number, 1
        )
    ).returns(mocked.serialize()).times(1);

    let _ = state_var.get_current_value_in_private();
}
