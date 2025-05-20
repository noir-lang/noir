use crate::shared_mutable::scheduled_value_change::ScheduledValueChange;

global TEST_DELAY: u32 = 200;

#[test]
unconstrained fn test_get_current_at() {
    let pre = 1;
    let post = 2;
    let block_of_change = 50;

    let value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(pre, post, block_of_change);

    assert_eq(value_change.get_current_at(0), pre);
    assert_eq(value_change.get_current_at(block_of_change - 1), pre);
    assert_eq(value_change.get_current_at(block_of_change), post);
    assert_eq(value_change.get_current_at(block_of_change + 1), post);
}

#[test]
unconstrained fn test_get_scheduled() {
    let pre = 1;
    let post = 2;
    let block_of_change = 50;

    let value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(pre, post, block_of_change);

    assert_eq(value_change.get_scheduled(), (post, block_of_change));
}

unconstrained fn assert_block_horizon_invariants(
    value_change: &mut ScheduledValueChange<Field>,
    historical_block_number: u32,
    block_horizon: u32,
) {
    // The current value should not change at the block horizon (but it might later).
    let current_at_historical = value_change.get_current_at(historical_block_number);
    assert_eq(current_at_historical, value_change.get_current_at(block_horizon));

    // The earliest a new change could be scheduled in would be the immediate next block to the historical one. This
    // should result in the new block of change landing *after* the block horizon, and the current value still not
    // changing at the previously determined block_horizon.
    let new = value_change.pre + value_change.post; // Make sure it's different to both pre and post
    value_change.schedule_change(
        new,
        historical_block_number + 1,
        TEST_DELAY,
        historical_block_number + 1 + TEST_DELAY,
    );

    assert(value_change.block_of_change > block_horizon);
    assert_eq(current_at_historical, value_change.get_current_at(block_horizon));
}

#[test]
unconstrained fn test_get_block_horizon_change_in_past() {
    let historical_block_number = 100;
    let block_of_change = 50;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(1, 2, block_of_change);

    let block_horizon = value_change.get_block_horizon(historical_block_number, TEST_DELAY);
    assert_eq(block_horizon, historical_block_number + TEST_DELAY);

    assert_block_horizon_invariants(&mut value_change, historical_block_number, block_horizon);
}

#[test]
unconstrained fn test_get_block_horizon_change_in_immediate_past() {
    let historical_block_number = 100;
    let block_of_change = 100;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(1, 2, block_of_change);

    let block_horizon = value_change.get_block_horizon(historical_block_number, TEST_DELAY);
    assert_eq(block_horizon, historical_block_number + TEST_DELAY);

    assert_block_horizon_invariants(&mut value_change, historical_block_number, block_horizon);
}

#[test]
unconstrained fn test_get_block_horizon_change_in_near_future() {
    let historical_block_number = 100;
    let block_of_change = 120;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(1, 2, block_of_change);

    // Note that this is the only scenario in which the block of change informs the block horizon.
    // This may result in privacy leaks when interacting with applications that have a scheduled change
    // in the near future.
    let block_horizon = value_change.get_block_horizon(historical_block_number, TEST_DELAY);
    assert_eq(block_horizon, block_of_change - 1);

    assert_block_horizon_invariants(&mut value_change, historical_block_number, block_horizon);
}

#[test]
unconstrained fn test_get_block_horizon_change_in_far_future() {
    let historical_block_number = 100;
    let block_of_change = 500;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(1, 2, block_of_change);

    let block_horizon = value_change.get_block_horizon(historical_block_number, TEST_DELAY);
    assert_eq(block_horizon, historical_block_number + TEST_DELAY);

    assert_block_horizon_invariants(&mut value_change, historical_block_number, block_horizon);
}

#[test]
unconstrained fn test_get_block_horizon_n0_delay() {
    let historical_block_number = 100;
    let block_of_change = 50;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(1, 2, block_of_change);

    let block_horizon = value_change.get_block_horizon(historical_block_number, 0);
    // Since the block horizon equals the historical block number, it is not possible to read the current value in
    // private since the transaction `max_block_number` property would equal an already mined block.
    assert_eq(block_horizon, historical_block_number);
}

#[test]
unconstrained fn test_schedule_change_before_change() {
    let pre = 1;
    let post = 2;
    let block_of_change = 500;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(pre, post, block_of_change);

    let new = 42;
    let current_block_number = block_of_change - 50;
    value_change.schedule_change(
        new,
        current_block_number,
        TEST_DELAY,
        current_block_number + TEST_DELAY,
    );

    // Because we re-schedule before the last scheduled change takes effect, the old `post` value is lost.
    assert_eq(value_change.pre, pre);
    assert_eq(value_change.post, new);
    assert_eq(value_change.block_of_change, current_block_number + TEST_DELAY);
}

#[test]
unconstrained fn test_schedule_change_after_change() {
    let pre = 1;
    let post = 2;
    let block_of_change = 500;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(pre, post, block_of_change);

    let new = 42;
    let current_block_number = block_of_change + 50;
    value_change.schedule_change(
        new,
        current_block_number,
        TEST_DELAY,
        current_block_number + TEST_DELAY,
    );

    assert_eq(value_change.pre, post);
    assert_eq(value_change.post, new);
    assert_eq(value_change.block_of_change, current_block_number + TEST_DELAY);
}

#[test]
unconstrained fn test_schedule_change_no_delay() {
    let pre = 1;
    let post = 2;
    let block_of_change = 500;

    let mut value_change: ScheduledValueChange<Field> =
        ScheduledValueChange::new(pre, post, block_of_change);

    let new = 42;
    let current_block_number = block_of_change + 50;
    value_change.schedule_change(new, current_block_number, 0, current_block_number);

    assert_eq(value_change.pre, post);
    assert_eq(value_change.post, new);
    assert_eq(value_change.block_of_change, current_block_number);
    assert_eq(value_change.get_current_at(current_block_number), new);
}
