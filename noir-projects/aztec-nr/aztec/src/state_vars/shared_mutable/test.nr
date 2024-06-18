use crate::{
    context::{PublicContext, PrivateContext}, state_vars::shared_mutable::shared_mutable::SharedMutable,
    test::helpers::test_environment::TestEnvironment
};

use dep::protocol_types::address::AztecAddress;

global new_value = 57;

global pre_delay = 20;
global post_delay = 15;

global storage_slot = 57;

global TEST_INITIAL_DELAY: u32 = 30;

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

// #[test]
// fn test_get_current_value_in_public_initial() {
//     let env = setup();
//     let state_var = in_public(env);

//     // 0 is the default empty value for a Field
//     assert_eq(state_var.get_current_value_in_public(), 0);
// }

// #[test]
// fn test_get_current_value_in_public_before_scheduled_change() {
//     let mut env = setup();
//     let state_var = in_public(env);

//     state_var.schedule_value_change(new_value);

//     let (_, block_of_change) = state_var.get_scheduled_value_in_public();

//     let original_value = 0;

//     // The current value has not changed
//     assert_eq(state_var.get_current_value_in_public(), original_value);

//     // The current value still does not change right before the block of change
//     env.advance_block_to(block_of_change - 1);
//     assert_eq(state_var.get_current_value_in_public(), original_value);
// }

// #[test]
// fn test_get_current_value_in_public_at_scheduled_change() {
//     let mut env = setup();
//     let state_var = in_public(env);

//     state_var.schedule_value_change(new_value);

//     let (_, block_of_change) = state_var.get_scheduled_value_in_public();

//     env.advance_block_to(block_of_change);
//     assert_eq(state_var.get_current_value_in_public(), new_value);
// }

// #[test]
// fn test_get_current_value_in_public_after_scheduled_change() {
//     let mut env = setup();
//     let state_var = in_public(env);

//     state_var.schedule_value_change(new_value);

//     let (_, block_of_change) = state_var.get_scheduled_value_in_public();

//     env.advance_block_to(block_of_change + 10);
//     assert_eq(state_var.get_current_value_in_public(), new_value);
// }

// #[test]
// fn test_get_current_value_in_private_before_change() {
//     let mut env = setup();

//     let public_state_var = in_public(env);
//     public_state_var.schedule_value_change(new_value);

//     let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

//     let schedule_block_number = env.block_number();

//     let private_state_var = in_private(&mut env, schedule_block_number);
//     assert_eq(private_state_var.get_current_value_in_private(), 0);
//     assert_eq(private_state_var.context.max_block_number.unwrap(), block_of_change - 1);
// }

// #[test]
// fn test_get_current_value_in_private_immediately_before_change() {
//     let mut env = setup();

//     let public_state_var = in_public(env);
//     public_state_var.schedule_value_change(new_value);

//     let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

//     let private_state_var = in_private(&mut env, block_of_change - 1);

//     assert_eq(private_state_var.get_current_value_in_private(), 0);
//     assert_eq(private_state_var.context.max_block_number.unwrap(), block_of_change - 1);
// }

// #[test]
// fn test_get_current_value_in_private_at_change() {
//     let mut env = setup();

//     let public_state_var = in_public(env);
//     public_state_var.schedule_value_change(new_value);

//     let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

//     let historical_block_number = block_of_change;
//     let private_state_var = in_private(&mut env, historical_block_number);
//     assert_eq(private_state_var.get_current_value_in_private(), new_value);
//     assert_eq(
//         private_state_var.context.max_block_number.unwrap(), historical_block_number + TEST_INITIAL_DELAY
//     );
// }

// #[test]
// fn test_get_current_value_in_private_after_change() {
//     let mut env = setup();

//     let public_state_var = in_public(env);
//     public_state_var.schedule_value_change(new_value);

//     let (_, block_of_change) = public_state_var.get_scheduled_value_in_public();

//     let historical_block_number = block_of_change + 10;
//     let private_state_var = in_private(&mut env, historical_block_number);
//     assert_eq(private_state_var.get_current_value_in_private(), new_value);
//     assert_eq(
//         private_state_var.context.max_block_number.unwrap(), historical_block_number + TEST_INITIAL_DELAY
//     );
// }

// #[test]
// fn test_get_current_delay_in_public() {
//     let (state_var, block_number) = setup();

//     // Uninitialized
//     mock_delay_change_read_uninitialized(state_var);
//     assert_eq(state_var.get_current_delay_in_public(), TEST_INITIAL_DELAY as u32);

//     // Change in the future, current value is pre
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
//     assert_eq(state_var.get_current_delay_in_public(), pre_delay as u32);

//     // Change in the current block, current value is post
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number);
//     assert_eq(state_var.get_current_delay_in_public(), post_delay as u32);

//     // Change in the past, current value is post
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
//     assert_eq(state_var.get_current_delay_in_public(), post_delay as u32);
// }

// #[test]
// fn test_get_scheduled_delay_in_public_before_change() {
//     let (state_var, block_number) = setup();

//     // Uninitialized
//     mock_delay_change_read_uninitialized(state_var);
//     assert_eq(state_var.get_scheduled_delay_in_public(), (TEST_INITIAL_DELAY as u32, 0));

//     // Change in the future, scheduled is post (always is)
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
//     assert_eq(state_var.get_scheduled_delay_in_public(), (post_delay as u32, (block_number + 1) as u32));

//     // Change in the current block, scheduled is post (always is)
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number);
//     assert_eq(state_var.get_scheduled_delay_in_public(), (post_delay as u32, block_number as u32));

//     // Change in the past, scheduled is post (always is)
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
//     assert_eq(state_var.get_scheduled_delay_in_public(), (post_delay as u32, (block_number - 1) as u32));
// }

// #[test]
// fn test_schedule_value_change_no_delay() {
//     let (state_var, block_number) = setup();

//     // Last value change was in the past
//     mock_value_change_read(state_var, pre_value, post_value, 0);

//     // Current delay is 0
//     mock_delay_change_read(state_var, 0, 0, block_number);

//     let write_mock = mock_value_change_write();

//     state_var.schedule_value_change(new_value);

//     // The new value has a block of change equal to the current block, i.e. it is the current value
//     assert_value_change_write(state_var, write_mock, post_value, new_value, block_number);
// }

// #[test]
// fn test_schedule_value_change_before_change_no_scheduled_delay() {
//     let (state_var, block_number) = setup();

//     // Value change in the future, delay change in the past
//     mock_value_and_delay_read(state_var, block_number + 1, block_number - 1);
//     let write_mock = mock_value_change_write();

//     state_var.schedule_value_change(new_value);

//     // The new scheduled value change replaces the old one, post delay (current) is used
//     assert_value_change_write(
//         state_var,
//         write_mock,
//         pre_value,
//         new_value,
//         block_number + post_delay
//     );
// }

// #[test]
// fn test_schedule_value_change_before_change_scheduled_delay() {
//     let (state_var, block_number) = setup();

//     // Value change in the future, delay change in the future
//     mock_value_and_delay_read(state_var, block_number + 1, block_number + 1);

//     let write_mock = mock_value_change_write();

//     state_var.schedule_value_change(new_value);

//     // The new scheduled value change replaces the old one, pre delay (current, not scheduled) is used
//     assert_value_change_write(
//         state_var,
//         write_mock,
//         pre_value,
//         new_value,
//         block_number + pre_delay
//     );
// }

// #[test]
// fn test_schedule_value_change_after_change_no_scheduled_delay() {
//     let (state_var, block_number) = setup();

//     // Value change in the past, delay change in the past
//     mock_value_and_delay_read(state_var, block_number - 1, block_number - 1);
//     let write_mock = mock_value_change_write();

//     state_var.schedule_value_change(new_value);

//     // The previous post value becomes the pre value, post delay (current) is used
//     assert_value_change_write(
//         state_var,
//         write_mock,
//         post_value,
//         new_value,
//         block_number + post_delay
//     );
// }

// #[test]
// fn test_schedule_value_change_after_change_scheduled_delay() {
//     let (state_var, block_number) = setup();

//     // Value change in the past, delay change in the future
//     mock_value_and_delay_read(state_var, block_number - 1, block_number + 1);

//     let write_mock = mock_value_change_write();

//     state_var.schedule_value_change(new_value);

//     // The previous post value becomes the pre value, pre delay (current, not scheduled) is used
//     assert_value_change_write(
//         state_var,
//         write_mock,
//         post_value,
//         new_value,
//         block_number + pre_delay
//     );
// }

// #[test]
// fn test_schedule_delay_increase_before_change() {
//     let (state_var, block_number) = setup();

//     // Delay change in future, current delay is pre
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
//     let write_mock = mock_delay_change_write();

//     let new_delay = pre_delay + 1;
//     state_var.schedule_delay_change(new_delay as u32);

//     // The previous scheduled change is lost, change is immediate (due to increase)
//     assert_delay_change_write(state_var, write_mock, pre_delay, new_delay, block_number);
// }

// #[test]
// fn test_schedule_delay_reduction_before_change() {
//     let (state_var, block_number) = setup();

//     // Delay change in future, current delay is pre
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number + 1);
//     let write_mock = mock_delay_change_write();

//     let new_delay = pre_delay - 1;
//     state_var.schedule_delay_change(new_delay as u32);

//     // The previous scheduled change is lost, change delay equals difference (due to reduction)
//     assert_delay_change_write(
//         state_var,
//         write_mock,
//         pre_delay,
//         new_delay,
//         block_number + pre_delay - new_delay
//     );
// }

// #[test]
// fn test_schedule_delay_increase_after_change() {
//     let (state_var, block_number) = setup();

//     // Delay change in the past, current delay is post
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
//     let write_mock = mock_delay_change_write();

//     let new_delay = post_delay + 1;
//     state_var.schedule_delay_change(new_delay as u32);

//     // The current value becomes pre, change is immediate (due to increase)
//     assert_delay_change_write(state_var, write_mock, post_delay, new_delay, block_number);
// }

// #[test]
// fn test_schedule_delay_reduction_after_change() {
//     let (state_var, block_number) = setup();

//     // Delay change in the past, current delay is post
//     mock_delay_change_read(state_var, pre_delay, post_delay, block_number - 1);
//     let write_mock = mock_delay_change_write();

//     let new_delay = post_delay - 1;
//     state_var.schedule_delay_change(new_delay as u32);

//     // The current value becomes pre, change delay equals difference (due to reduction)
//     assert_delay_change_write(
//         state_var,
//         write_mock,
//         post_delay,
//         new_delay,
//         block_number + post_delay - new_delay
//     );
// }