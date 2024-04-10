use dep::protocol_types::traits::{Serialize, Deserialize, FromField, ToField};

// This data structure is used by SharedMutable to represent a value that changes from `pre` to `post` at some block 
// called the `block_of_change`. The value can only be made to change by scheduling a change event at some future block
// of change after some delay measured in blocks has elapsed. This means that at any given block number we know both the 
// current value and the smallest block number at which the value might change - this is called the 'block horizon'.
//
// The delay being a type parameter instead of a struct field is an implementation detail, and is due to a number of
// reasons:
//  - we want to serialize and deserialize this object in order to store it in public storage, but we don't want to
//    include the delay there because it is immutable
//  - because of how aztec-nr state variables are declared, having a type with some immutable property is better 
//    expressed via types, since they are always constructed with the same `::new(context, storage_slot)` function.
struct ScheduledValueChange<T, DELAY> {
    pre: T,
    post: T,
    block_of_change: u32,
    // The _dummy variable forces DELAY to be interpreted as a numberic value. This is a workaround to
    // https://github.com/noir-lang/noir/issues/4633. Remove once resolved.
    _dummy: [Field; DELAY],
}

impl<T, DELAY> ScheduledValueChange<T, DELAY> {
    pub fn new(pre: T, post: T, block_of_change: u32) -> Self {
        Self { pre, post, block_of_change, _dummy: [0; DELAY] }
    }

    /// Returns the value stored in the data structure at a given block. This function can be called both in public 
    /// (where `block_number` is simply the current block number, i.e. the number of the block in which the current 
    /// transaction will be included) and in private (where `block_number` is the historical block number that is used 
    /// to construct the proof).
    /// Reading in private is only safe if the transaction's `max_block_number` property is set to a value lower or 
    /// equal to the block horizon (see `get_block_horizon()`).
    pub fn get_current_at(self, block_number: u32) -> T {
        // The post value becomes the current one at the block of change. This means different things in each realm:
        // - in public, any transaction that is included in the block of change will use the post value
        // - in private, any transaction that includes the block of change as part of the historical state will use the
        //   post value (barring any follow-up changes)

        if block_number < self.block_of_change {
            self.pre
        } else {
            self.post
        }
    }

    /// Returns the scheduled change, i.e. the post-change value and the block at which it will become the current 
    /// value. Note that this block may be in the past if the change has already taken place.
    /// Additionally, further changes might be later scheduled, potentially canceling the one returned by this function.
    pub fn get_scheduled(self) -> (T, u32) {
        (self.post, self.block_of_change)
    }

    /// Returns the largest block number at which the value returned by `get_current_at` is known to remain the current
    /// value. This value is only meaningful in private when constructing a proof at some `historical_block_number`, 
    /// since due to its asynchronous nature private execution cannot know about any later scheduled changes.
    /// The value returned by `get_current_at` in private when called with a historical block number is only safe to use
    /// if the transaction's `max_block_number` property is set to a value lower or equal to the block horizon computed
    /// using the same historical block number.
    pub fn get_block_horizon(self, historical_block_number: u32) -> u32 {
        // The block horizon is the very last block in which the current value is known. Any block past the horizon 
        // (i.e. with a block number larger than the block horizon) may have a different current value. Reading the
        // current value in private typically requires constraining the maximum valid block number to be equal to the
        // block horizon.

        if historical_block_number >= self.block_of_change {
            // Once the block of change has been mined, the current value (post) will not change unless a new value
            // change is scheduled. This did not happen at the historical block number (or else it would not be
            // greater or equal to the block of change), and therefore could only happen after the historical block
            // number. The earliest would be the immediate next block, and so the smallest possible next block of change
            // equals `historical_block_number + 1 + DELAY`. Our block horizon is simply the previous block to that one.
            //
            //      block of      historical        
            //       change         block            block horizon
            //   =======|=============N===================H===========>
            //                         ^                   ^
            //                         ---------------------
            //                                delay

            historical_block_number + DELAY
        } else {
            // If the block of change has not yet been mined however, then there are two possible scenarios.
            //   a) It could be so far into the future that the block horizon is actually determined by the delay,
            //      because a new change could be scheduled and take place _before_ the currently scheduled one. This is
            //      similar to the scenario where the block of change is in the past: the time horizon is the block
            //      prior to the earliest one in which a new block of change might land.
            // 
            //         historical                         
            //           block                        block horizon      block of change
            //        =====N=================================H=================|=========>
            //              ^                                 ^
            //              |                                 |   
            //              -----------------------------------
            //                           delay
            //
            //   b) It could be fewer than `delay` blocks away from the historical block number, in which case it would
            //      become the limiting factor for the time horizon, which would be the block right before the block of
            //      change (since by definition the value changes at the block of change).
            //
            //           historical                         block horizon
            //             block     block of change       if not scheduled
            //        =======N=============|===================H=================>
            //                ^           ^                     ^
            //                |     actual horizon              |   
            //                -----------------------------------
            //                             delay
            //      
            // Note that the current implementation does not allow the caller to set the block of change to an arbitrary
            // value, and therefore scenario a) is not currently possible. However implementing #5501 would allow for 
            // this to happen.

            // Because historical_block_number < self.block_of_change, then block_of_change > 0 and we can safely
            // subtract 1.
            min(self.block_of_change - 1, historical_block_number + DELAY)
        }
    }

    /// Mutates a scheduled value change by scheduling a change at the current block number. This function is only
    /// meaningful when called in public with the current block number.
    pub fn schedule_change(&mut self, new_value: T, current_block_number: u32) {
        self.pre = self.get_current_at(current_block_number);
        self.post = new_value;
        // TODO: make this configurable 
        // https://github.com/AztecProtocol/aztec-packages/issues/5501
        self.block_of_change = current_block_number + DELAY;
    }
}

impl<T, DELAY> Serialize<3> for ScheduledValueChange<T, DELAY> {
    fn serialize(self) -> [Field; 3] where T: ToField {
        [self.pre.to_field(), self.post.to_field(), self.block_of_change.to_field()]
    }
}

impl<T, DELAY> Deserialize<3> for ScheduledValueChange<T, DELAY> {
  fn deserialize(input: [Field; 3]) -> Self  where T: FromField {
    Self { 
        pre: FromField::from_field(input[0]), 
        post: FromField::from_field(input[1]), 
        block_of_change: FromField::from_field(input[2]), 
        _dummy: [0; DELAY] 
    }
  }
}

fn min(lhs: u32, rhs: u32) -> u32 {
    if lhs < rhs { lhs } else { rhs }
}

#[test]
fn test_min() {
    assert(min(3, 5) == 3);
    assert(min(5, 3) == 3);
    assert(min(3, 3) == 3);
}

mod test {
    use crate::state_vars::shared_mutable::scheduled_value_change::ScheduledValueChange;

    global TEST_DELAY = 200;

    #[test]
    fn test_get_current_at() {
        let pre = 1;
        let post = 2;
        let block_of_change = 50;

        let value: ScheduledValueChange<Field, TEST_DELAY> = ScheduledValueChange::new(pre, post, block_of_change);

        assert_eq(value.get_current_at(0), pre);
        assert_eq(value.get_current_at(block_of_change - 1), pre);
        assert_eq(value.get_current_at(block_of_change), post);
        assert_eq(value.get_current_at(block_of_change + 1), post);
    }

    #[test]
    fn test_get_scheduled() {
        let pre = 1;
        let post = 2;
        let block_of_change = 50;

        let value: ScheduledValueChange<Field, TEST_DELAY> = ScheduledValueChange::new(pre, post, block_of_change);

        assert_eq(value.get_scheduled(), (post, block_of_change));
    }

    fn assert_block_horizon_invariants(
        value: &mut ScheduledValueChange<Field, TEST_DELAY>,
        historical_block_number: u32,
        block_horizon: u32
    ) {
        // The current value should not change at the block horizon (but it might later).
        let current_at_historical = value.get_current_at(historical_block_number);
        assert_eq(current_at_historical, value.get_current_at(block_horizon));

        // The earliest a new change could be scheduled in would be the immediate next block to the historical one. This
        // should result in the new block of change landing *after* the block horizon, and the current value still not
        // changing at the previously determined block_horizon.

        let new = value.pre + value.post; // Make sure it's different to both pre and post
        value.schedule_change(new, historical_block_number + 1);

        assert(value.block_of_change > block_horizon);
        assert_eq(current_at_historical, value.get_current_at(block_horizon));
    }

    #[test]
    fn test_get_block_horizon_change_in_past() {
        let historical_block_number = 100;
        let block_of_change = 50;

        let mut value: ScheduledValueChange<Field, TEST_DELAY> = ScheduledValueChange::new(1, 2, block_of_change);

        let block_horizon = value.get_block_horizon(historical_block_number);
        assert_eq(block_horizon, historical_block_number + TEST_DELAY);

        assert_block_horizon_invariants(&mut value, historical_block_number, block_horizon);
    }

    #[test]
    fn test_get_block_horizon_change_in_immediate_past() {
        let historical_block_number = 100;
        let block_of_change = 100;

        let mut value: ScheduledValueChange<Field, TEST_DELAY> = ScheduledValueChange::new(1, 2, block_of_change);

        let block_horizon = value.get_block_horizon(historical_block_number);
        assert_eq(block_horizon, historical_block_number + TEST_DELAY);

        assert_block_horizon_invariants(&mut value, historical_block_number, block_horizon);
    }

    #[test]
    fn test_get_block_horizon_change_in_near_future() {
        let historical_block_number = 100;
        let block_of_change = 120;

        let mut value: ScheduledValueChange<Field, TEST_DELAY> = ScheduledValueChange::new(1, 2, block_of_change);

        // Note that this is the only scenario in which the block of change informs the block horizon.
        // This may result in privacy leaks when interacting with applications that have a scheduled change
        // in the near future.
        let block_horizon = value.get_block_horizon(historical_block_number);
        assert_eq(block_horizon, block_of_change - 1);

        assert_block_horizon_invariants(&mut value, historical_block_number, block_horizon);
    }

    #[test]
    fn test_get_block_horizon_change_in_far_future() {
        let historical_block_number = 100;
        let block_of_change = 500;

        let mut value: ScheduledValueChange<Field, TEST_DELAY> = ScheduledValueChange::new(1, 2, block_of_change);

        let block_horizon = value.get_block_horizon(historical_block_number);
        assert_eq(block_horizon, historical_block_number + TEST_DELAY);

        assert_block_horizon_invariants(&mut value, historical_block_number, block_horizon);
    }

    #[test]
    fn test_schedule_change_before_prior_change() {
        let pre = 1;
        let post = 2;
        let block_of_change = 500;

        let mut value: ScheduledValueChange<Field, TEST_DELAY> = ScheduledValueChange::new(pre, post, block_of_change);

        let new = 42;
        let current_block_number = block_of_change - 50;
        value.schedule_change(new, current_block_number);

        // Because we re-schedule before the last scheduled change takes effect, the old `post` value is lost.
        assert_eq(value.pre, pre);
        assert_eq(value.post, new);
        assert_eq(value.block_of_change, current_block_number + TEST_DELAY);
    }

    #[test]
    fn test_schedule_change_after_prior_change() {
        let pre = 1;
        let post = 2;
        let block_of_change = 500;

        let mut value: ScheduledValueChange<Field, TEST_DELAY>  = ScheduledValueChange::new(pre, post, block_of_change);

        let new = 42;
        let current_block_number = block_of_change + 50;
        value.schedule_change(new, current_block_number);

        assert_eq(value.pre, post);
        assert_eq(value.post, new);
        assert_eq(value.block_of_change, current_block_number + TEST_DELAY);
    }
}
