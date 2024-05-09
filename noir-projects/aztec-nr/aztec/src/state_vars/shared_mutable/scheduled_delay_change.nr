use dep::protocol_types::traits::{Serialize, Deserialize, FromField, ToField};
use dep::std::cmp::min;

// This data structure is used by SharedMutable to store the minimum delay with which a ScheduledValueChange object can
// schedule a change.
// This delay is initally equal to INITIAL_DELAY, and can be safely mutated to any other value over time. This mutation 
// is performed via `schedule_change` in order to satisfy ScheduleValueChange constraints: if e.g. we allowed for the 
// delay to be decreased immediately then it'd be possible for the state variable to schedule a value change with a 
// reduced delay, invalidating prior private reads.
struct ScheduledDelayChange<INITIAL_DELAY> {
    // Both pre and post are stored in public storage, so by default they are zeroed. By wrapping them in an Option, 
    // they default to Option::none(), which we detect and replace with INITIAL_DELAY. The end result is that a
    // ScheduledDelayChange that has not been initialized has a delay equal to INITIAL_DELAY, which is the desired
    // effect. Once initialized, the Option will never be none again.
    pre: Option<u32>,
    post: Option<u32>,
    // Block at which `post` value is used instead of `pre`
    block_of_change: u32,
    // The _dummy variable forces INITIAL_DELAY to be interpreted as a numeric value. This is a workaround to
    // https://github.com/noir-lang/noir/issues/4633. Remove once resolved.
    _dummy: [Field; INITIAL_DELAY],
}

impl<INITIAL_DELAY> ScheduledDelayChange<INITIAL_DELAY> {
    pub fn new(pre: Option<u32>, post: Option<u32>, block_of_change: u32) -> Self {
        Self { pre, post, block_of_change, _dummy: [0; INITIAL_DELAY] }
    }

    /// Returns the current value of the delay stored in the data structure.
    /// This function only returns a meaningful value when called in public with the current block number - for
    /// historical private reads use `get_effective_minimum_delay_at` instead.
    pub fn get_current(self, current_block_number: u32) -> u32 {
        // The post value becomes the current one at the block of change, so any transaction that is included in the
        // block of change will use the post value.

        if current_block_number < self.block_of_change {
            self.pre.unwrap_or(INITIAL_DELAY)
        } else {
            self.post.unwrap_or(INITIAL_DELAY)
        }
    }

    /// Returns the scheduled change, i.e. the post-change delay and the block at which it will become the current
    /// delay. Note that this block may be in the past if the change has already taken place.
    /// Additionally, further changes might be later scheduled, potentially canceling the one returned by this function.
    pub fn get_scheduled(self) -> (u32, u32) {
        (self.post.unwrap_or(INITIAL_DELAY), self.block_of_change)
    }

    /// Mutates the delay change by scheduling a change at the current block number. This function is only meaningful
    /// when called in public with the current block number.
    /// The block at which the new delay will become effective is determined automatically:
    ///  - when increasing the delay, the change is effective immediately
    ///  - when reducing the delay, the change will take effect after a delay equal to the difference between old and
    ///    new delay. For example, if reducing from 3 days to 1 day, the reduction will be scheduled to happen after 2
    ///    days.
    pub fn schedule_change(&mut self, new: u32, current_block_number: u32) {
        let current = self.get_current(current_block_number);

        // When changing the delay value we must ensure that it is not possible to produce a value change with a delay
        // shorter than the current one.
        let blocks_until_change = if new > current {
            // Increasing the delay value can therefore be done immediately: this does not invalidate prior contraints
            // about how quickly a value might be changed (indeed it strengthens them).
            0
        } else {
            // Decreasing the delay requires waiting for the difference between current and new delay in order to ensure
            // that overall the current delay is respected.
            //
            //      current                    delay              earliest value block of change
            //       block                 block of change         if delay remained unchanged
            //  =======N=========================|================================X=================>
            //         ^                         ^                                ^
            //         |-------------------------|--------------------------------|
            //         |   blocks until change               new delay            |
            //         ------------------------------------------------------------
            //                            current delay
            current - new
        };

        self.pre = Option::some(current);
        self.post = Option::some(new);
        self.block_of_change = current_block_number + blocks_until_change;
    }

    /// Returns the minimum delay before a value might mutate due to a scheduled change, from the perspective of some
    /// historical block number. It only returns a meaningful value when called in private with historical blocks. This 
    /// function can be used alongside `ScheduledValueChange.get_block_horizon` to properly constrain the
    /// `max_block_number` transaction property when reading mutable shared state.
    /// This value typically equals the current delay at the block following the historical one (the earliest one in
    /// which a value change could be scheduled), but it also considers scenarios in which a delay reduction is 
    /// scheduled to happen in the near future, resulting in a way to schedule a change with an overall delay lower than
    /// the current one.
    pub fn get_effective_minimum_delay_at(self, historical_block_number: u32) -> u32 {
        if self.block_of_change <= historical_block_number {
            // If no delay changes were scheduled, then the delay value at the historical block (post) is guaranteed to
            // hold due to how further delay changes would be scheduled by `schedule_change`.
            self.post.unwrap_or(INITIAL_DELAY)
        } else {
            // If a change is scheduled, then the effective delay might be lower than the current one (pre). At the
            // block of change the current delay will be the scheduled one, with an overall delay from the historical
            // block number equal to the number of blocks until the change plus the new delay. If this value is lower
            // than the current delay, then that is the effective minimum delay.
            //
            //       historical
            //         block                  delay                  actual earliest value
            //           v                block of change              block of change
            //  =========NS=====================|=============================X===========Y=====>
            //            ^                     ^                             ^           ^
            //     earliest block in            |                             |           |
            //   which to schedule change       |                             |           |
            //           |                      |                             |           |
            //           |----------------------|------------------------------           |
            //           |         blocks                  new delay                      |
            //           |      until change                                              |
            //           |                                                                |
            //           |----------------------------------------------------------------|
            //                        current delay at the earliest block in 
            //                             which to scheduled value change

            let blocks_until_change = self.block_of_change - (historical_block_number + 1);

            min(
                self.pre.unwrap_or(INITIAL_DELAY),
                blocks_until_change + self.post.unwrap_or(INITIAL_DELAY)
            )
        }
    }
}

impl<INITIAL_DELAY> Serialize<1> for ScheduledDelayChange<INITIAL_DELAY> {
    fn serialize(self) -> [Field; 1] {
        // We pack all three u32 values into a single U128, which is made up of two u64 limbs.
        // Low limb: [ pre_inner: u32 | post_inner: u32 ]
        // High limb: [ empty | pre_is_some: u8 |  post_is_some: u8 | block_of_change: u32 ]

        let lo = ((self.pre.unwrap_unchecked() as u64) * (1 << 32))
            + (self.post.unwrap_unchecked() as u64);

        let hi = (self.pre.is_some() as u64) * (1 << 33) 
            + (self.post.is_some() as u64 * (1 << 32)) 
            + self.block_of_change as u64;

        let packed = U128::from_u64s_le(lo, hi);

        [packed.to_integer()]
    }
}

impl<INITIAL_DELAY> Deserialize<1> for ScheduledDelayChange<INITIAL_DELAY> {
    fn deserialize(input: [Field; 1]) -> Self {
        let packed = U128::from_integer(input[0]);

        // We use division and modulo to clear the bits that correspond to other values when unpacking.

        let pre_is_some = ((packed.hi as u64) / (1 << 33)) as bool;
        let pre_inner = ((packed.lo as u64) / (1 << 32)) as u32;

        let post_is_some = (((packed.hi as u64) / (1 << 32)) % (1 << 1)) as bool;
        let post_inner = ((packed.lo as u64) % (1 << 32)) as u32;

        let block_of_change = ((packed.hi as u64) % (1 << 32)) as u32;

        Self {
            pre: if pre_is_some { Option::some(pre_inner) } else { Option::none() },
            post: if post_is_some { Option::some(post_inner) } else { Option::none() },
            block_of_change,
            _dummy: [0; INITIAL_DELAY],
        }
    }
}

mod test {
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
}
