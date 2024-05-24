use crate::state_vars::shared_mutable::scheduled_delay_change::ScheduledDelayChange;

global TEST_INITIAL_DELAY = 13;

fn assert_equal_after_conversion(original: ScheduledDelayChange<TEST_INITIAL_DELAY>) {
    // We have to do explicit type annotations because Noir lacks turbofish support.
    // TODO: improve syntax once https://github.com/noir-lang/noir/issues/4710 is implemented.
    let converted: ScheduledDelayChange<TEST_INITIAL_DELAY> = ScheduledDelayChange::deserialize((original).serialize());

    assert_eq(original.pre, converted.pre);
    assert_eq(original.post, converted.post);
    assert_eq(original.block_of_change, converted.block_of_change);
}

#[test]
fn test_serde() {
    let pre = 1;
    let post = 2;
    let block_of_change = 50;

    assert_equal_after_conversion(ScheduledDelayChange::new(Option::some(pre), Option::some(post), block_of_change));
    assert_equal_after_conversion(ScheduledDelayChange::new(Option::some(pre), Option::none(), block_of_change));
    assert_equal_after_conversion(ScheduledDelayChange::new(Option::none(), Option::some(post), block_of_change));
    assert_equal_after_conversion(ScheduledDelayChange::new(Option::none(), Option::none(), block_of_change));
}

#[test]
fn test_serde_large_values() {
    let max_u32 = (1 << 32) - 1;

    let pre = max_u32 as u32;
    let post = (max_u32 - 1) as u32;
    let block_of_change = (max_u32 - 2) as u32;

    assert_equal_after_conversion(ScheduledDelayChange::new(Option::some(pre), Option::some(post), block_of_change));
    assert_equal_after_conversion(ScheduledDelayChange::new(Option::some(pre), Option::none(), block_of_change));
    assert_equal_after_conversion(ScheduledDelayChange::new(Option::none(), Option::some(post), block_of_change));
    assert_equal_after_conversion(ScheduledDelayChange::new(Option::none(), Option::none(), block_of_change));
}

fn get_non_initial_delay_change(
    pre: u32,
    post: u32,
    block_of_change: u32
) -> ScheduledDelayChange<TEST_INITIAL_DELAY> {
    ScheduledDelayChange::new(Option::some(pre), Option::some(post), block_of_change)
}

fn get_initial_delay_change() -> ScheduledDelayChange<TEST_INITIAL_DELAY> {
    ScheduledDelayChange::deserialize([0])
}

#[test]
fn test_get_current() {
    let pre = 1;
    let post = 2;
    let block_of_change = 50;

    let delay_change = get_non_initial_delay_change(pre, post, block_of_change);

    assert_eq(delay_change.get_current(0), pre);
    assert_eq(delay_change.get_current(block_of_change - 1), pre);
    assert_eq(delay_change.get_current(block_of_change), post);
    assert_eq(delay_change.get_current(block_of_change + 1), post);
}

#[test]
fn test_get_current_initial() {
    let delay_change = get_initial_delay_change();

    assert_eq(delay_change.get_current(0), TEST_INITIAL_DELAY);
    assert_eq(delay_change.get_current(1), TEST_INITIAL_DELAY);
}

#[test]
fn test_get_scheduled() {
    let pre = 1;
    let post = 2;
    let block_of_change = 50;

    let delay_change = get_non_initial_delay_change(pre, post, block_of_change);

    assert_eq(delay_change.get_scheduled(), (post, block_of_change));
}

#[test]
fn test_get_scheduled_initial() {
    let delay_change = get_initial_delay_change();

    assert_eq(delay_change.get_scheduled(), (TEST_INITIAL_DELAY, 0));
}

#[test]
fn test_schedule_change_to_shorter_delay_before_change() {
    let pre = 15;
    let post = 25;
    let block_of_change = 500;

    let new = 10;
    let current_block_number = block_of_change - 50;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);
    delay_change.schedule_change(new, current_block_number);

    // Because we re-schedule before the last scheduled change takes effect, the old `post` value is lost. The
    // schedule time is determined by the difference between the current value (pre) and new delay.
    assert_eq(delay_change.pre.unwrap(), pre);
    assert_eq(delay_change.post.unwrap(), new);
    assert_eq(delay_change.block_of_change, current_block_number + pre - new);
}

#[test]
fn test_schedule_change_to_shorter_delay_after_change() {
    let pre = 15;
    let post = 25;
    let block_of_change = 500;

    let new = 10;
    let current_block_number = block_of_change + 50;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);
    delay_change.schedule_change(new, current_block_number);

    // The schedule time is determined by the different between the current value (ex post, now pre) and new delay.
    assert_eq(delay_change.pre.unwrap(), post);
    assert_eq(delay_change.post.unwrap(), new);
    assert_eq(delay_change.block_of_change, current_block_number + post - new);
}

#[test]
fn test_schedule_change_to_shorter_delay_from_initial() {
    let new = TEST_INITIAL_DELAY - 1;
    let current_block_number = 50;

    let mut delay_change = get_initial_delay_change();
    delay_change.schedule_change(new, current_block_number);

    // Like in the after change scenario, the schedule time is determined by the difference between the current value
    // (initial) and new delay.
    assert_eq(delay_change.pre.unwrap(), TEST_INITIAL_DELAY);
    assert_eq(delay_change.post.unwrap(), new);
    assert_eq(delay_change.block_of_change, current_block_number + TEST_INITIAL_DELAY - new);
}

#[test]
fn test_schedule_change_to_longer_delay_before_change() {
    let pre = 15;
    let post = 25;
    let block_of_change = 500;

    let new = 40;
    let current_block_number = block_of_change - 50;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);
    delay_change.schedule_change(new, current_block_number);

    // Because we re-schedule before the last scheduled change takes effect, the old `post` value is lost. The
    // change is effective immediately because the new delay is longer than the current one.
    assert_eq(delay_change.pre.unwrap(), pre);
    assert_eq(delay_change.post.unwrap(), new);
    assert_eq(delay_change.block_of_change, current_block_number);
    assert_eq(delay_change.get_current(current_block_number), new);
}

#[test]
fn test_schedule_change_to_longer_delay_after_change() {
    let pre = 15;
    let post = 25;
    let block_of_change = 500;

    let new = 40;
    let current_block_number = block_of_change + 50;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);
    delay_change.schedule_change(new, current_block_number);

    // Change is effective immediately because the new delay is longer than the current one.
    assert_eq(delay_change.pre.unwrap(), post);
    assert_eq(delay_change.post.unwrap(), new);
    assert_eq(delay_change.block_of_change, current_block_number);
    assert_eq(delay_change.get_current(current_block_number), new);
}

#[test]
fn test_schedule_change_to_longer_delay_from_initial() {
    let new = TEST_INITIAL_DELAY + 1;
    let current_block_number = 50;

    let mut delay_change = get_initial_delay_change();
    delay_change.schedule_change(new, current_block_number);

    // Like in the after change scenario, change is effective immediately because the new delay is longer than the 
    // current one.
    assert_eq(delay_change.pre.unwrap(), TEST_INITIAL_DELAY);
    assert_eq(delay_change.post.unwrap(), new);
    assert_eq(delay_change.block_of_change, current_block_number);
    assert_eq(delay_change.get_current(current_block_number), new);
}

fn assert_effective_minimum_delay_invariants<INITIAL_DELAY>(
    delay_change: &mut ScheduledDelayChange<INITIAL_DELAY>,
    historical_block_number: u32,
    effective_minimum_delay: u32
) {
    // The effective minimum delays guarantees the earliest block in which a scheduled value change could be made
    // effective. No action, even if executed immediately after the historical block, should result in a scheduled
    // value change having a block of change lower than this.
    let expected_earliest_value_change_block = historical_block_number + 1 + effective_minimum_delay;

    if delay_change.block_of_change > historical_block_number {
        // If a delay change is already scheduled to happen in the future, we then must consider the scenario in
        // which a value change is scheduled to occur right as the delay changes and becomes the current one.
        let delay_change_block = delay_change.block_of_change;

        let value_change_block = delay_change_block + delay_change.get_current(delay_change_block);
        assert(expected_earliest_value_change_block <= value_change_block);
    }

    // Another possibility would be to schedule a value change immediately after the historical block.
    let change_schedule_block = historical_block_number + 1;
    let value_change_block = change_schedule_block + delay_change.get_current(change_schedule_block);
    assert(expected_earliest_value_change_block <= value_change_block);

    // Finally, a delay reduction could be scheduled immediately after the historical block. We reduce the delay to 
    // zero, which means that at the delay block of change there'll be no delay and a value change could be 
    // performed immediately then.
    delay_change.schedule_change(0, historical_block_number + 1);
    assert(expected_earliest_value_change_block <= delay_change.block_of_change);
}

#[test]
fn test_get_effective_delay_at_before_change_in_far_future() {
    let pre = 15;
    let post = 25;
    let block_of_change = 500;

    let historical_block_number = 200;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);

    // The scheduled delay change is far into the future (further than the current delay is), so it doesn't affect
    // the effective delay, which is simply the current one (pre).
    let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
    assert_eq(effective_minimum_delay, pre);

    assert_effective_minimum_delay_invariants(
        &mut delay_change,
        historical_block_number,
        effective_minimum_delay
    );
}

#[test]
fn test_get_effective_delay_at_before_change_to_long_delay() {
    let pre = 15;
    let post = 25;
    let block_of_change = 500;

    let historical_block_number = 495;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);

    // The scheduled delay change will be effective soon (it's fewer blocks away than the current delay), but due to
    // it being larger than the current one it doesn't affect the effective delay, which is simply the current one
    // (pre).
    let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
    assert_eq(effective_minimum_delay, pre);

    assert_effective_minimum_delay_invariants(
        &mut delay_change,
        historical_block_number,
        effective_minimum_delay
    );
}

#[test]
fn test_get_effective_delay_at_before_near_change_to_short_delay() {
    let pre = 15;
    let post = 3;
    let block_of_change = 500;

    let historical_block_number = 495;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);

    // The scheduled delay change will be effective soon (it's fewer blocks away than the current delay), and it's
    // changing to a value smaller than the current one. This means that at the block of change the delay will be
    // reduced, and a delay change would be scheduled there with an overall delay lower than the current one.
    // The effective delay therefore is the new delay plus the number of blocks that need to elapse until it becomes
    // effective (i.e. until the block of change).
    let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
    assert_eq(effective_minimum_delay, post + block_of_change - (historical_block_number + 1));

    assert_effective_minimum_delay_invariants(
        &mut delay_change,
        historical_block_number,
        effective_minimum_delay
    );
}

#[test]
fn test_get_effective_delay_at_after_change() {
    let pre = 15;
    let post = 25;
    let block_of_change = 500;

    let historical_block_number = block_of_change + 50;

    let mut delay_change = get_non_initial_delay_change(pre, post, block_of_change);

    // No delay change is scheduled, so the effective delay is simply the current one (post).
    let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
    assert_eq(effective_minimum_delay, post);

    assert_effective_minimum_delay_invariants(
        &mut delay_change,
        historical_block_number,
        effective_minimum_delay
    );
}

#[test]
fn test_get_effective_delay_at_initial() {
    let mut delay_change = get_initial_delay_change();

    let historical_block_number = 200;

    // Like in the after change scenario, no delay change is scheduled, so the effective delay is simply the current 
    // one (initial).
    let effective_minimum_delay = delay_change.get_effective_minimum_delay_at(historical_block_number);
    assert_eq(effective_minimum_delay, TEST_INITIAL_DELAY);

    assert_effective_minimum_delay_invariants(
        &mut delay_change,
        historical_block_number,
        effective_minimum_delay
    );
}
