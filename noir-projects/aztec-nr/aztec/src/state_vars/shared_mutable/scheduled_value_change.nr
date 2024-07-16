use dep::protocol_types::traits::{Serialize, Deserialize, FromField, ToField};
use std::cmp::min;

mod test;

// This data structure is used by SharedMutable to represent a value that changes from `pre` to `post` at some block
// called the `block_of_change`. The value can only be made to change by scheduling a change event at some future block
// of change after some minimum delay measured in blocks has elapsed. This means that at any given block number we know
// both the current value and the smallest block number at which the value might change - this is called the
// 'block horizon'.
struct ScheduledValueChange<T> {
    pre: T,
    post: T,
    // Block at which `post` value is used instead of `pre`
    block_of_change: u32,
}

impl<T> ScheduledValueChange<T> {
    pub fn new(pre: T, post: T, block_of_change: u32) -> Self {
        Self { pre, post, block_of_change }
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
    /// The caller of this function must know how quickly the value can change due to a scheduled change in the form of
    /// `minimum_delay`. If the delay itself is immutable, then this is just its duration. If the delay is mutable
    /// however, then this value is the 'effective minimum delay' (obtained by calling
    /// `ScheduledDelayChange.get_effective_minimum_delay_at`), which equals the minimum number of blocks that need to
    /// elapse from the next block until the value changes, regardless of further delay changes.
    /// The value returned by `get_current_at` in private when called with a historical block number is only safe to use
    /// if the transaction's `max_block_number` property is set to a value lower or equal to the block horizon computed
    /// using the same historical block number.
    pub fn get_block_horizon(self, historical_block_number: u32, minimum_delay: u32) -> u32 {
        // The block horizon is the very last block in which the current value is known. Any block past the horizon
        // (i.e. with a block number larger than the block horizon) may have a different current value. Reading the
        // current value in private typically requires constraining the maximum valid block number to be equal to the
        // block horizon.

        if historical_block_number >= self.block_of_change {
            // Once the block of change has been mined, the current value (post) will not change unless a new value
            // change is scheduled. This did not happen at the historical block number (or else it would not be
            // greater or equal to the block of change), and therefore could only happen after the historical block
            // number. The earliest would be the immediate next block, and so the smallest possible next block of change
            // equals `historical_block_number + 1 + minimum_delay`. Our block horizon is simply the previous block to
            // that one.
            //
            //      block of      historical
            //       change         block            block horizon
            //   =======|=============N===================H===========>
            //                         ^                   ^
            //                         ---------------------
            //                             minimum delay

            historical_block_number + minimum_delay
        } else {
            // If the block of change has not yet been mined however, then there are two possible scenarios.
            //   a) It could be so far into the future that the block horizon is actually determined by the minimum
            //      delay, because a new change could be scheduled and take place _before_ the currently scheduled one.
            //      This is similar to the scenario where the block of change is in the past: the time horizon is the
            //      block prior to the earliest one in which a new block of change might land.
            //
            //         historical
            //           block                        block horizon      block of change
            //        =====N=================================H=================|=========>
            //              ^                                 ^
            //              |                                 |
            //              -----------------------------------
            //                        minimum delay
            //
            //   b) It could be fewer than `minimum_delay` blocks away from the historical block number, in which case
            //      the block of change would become the limiting factor for the time horizon, which would equal the
            //      block right before the block of change (since by definition the value changes at the block of
            //      change).
            //
            //           historical                         block horizon
            //             block     block of change       if not scheduled
            //        =======N=============|===================H=================>
            //                ^           ^                     ^
            //                |     actual horizon              |
            //                -----------------------------------
            //                          minimum delay
            //
            // Note that the current implementation does not allow the caller to set the block of change to an arbitrary
            // value, and therefore scenario a) is not currently possible. However implementing #5501 would allow for
            // this to happen.

            // Because historical_block_number < self.block_of_change, then block_of_change > 0 and we can safely
            // subtract 1.
            min(
                self.block_of_change - 1,
                historical_block_number + minimum_delay
            )
        }
    }

    /// Mutates the value by scheduling a change at the current block number. This function is only meaningful when
    /// called in public with the current block number.
    pub fn schedule_change(
        &mut self,
        new_value: T,
        current_block_number: u32,
        minimum_delay: u32,
        block_of_change: u32
    ) {
        assert(block_of_change >= current_block_number + minimum_delay);

        self.pre = self.get_current_at(current_block_number);
        self.post = new_value;
        self.block_of_change = block_of_change;
    }
}

impl<T> Serialize<3> for ScheduledValueChange<T> where T: ToField {
    fn serialize(self) -> [Field; 3] {
        [self.pre.to_field(), self.post.to_field(), self.block_of_change.to_field()]
    }
}

impl<T> Deserialize<3> for ScheduledValueChange<T> where T: FromField {
  fn deserialize(input: [Field; 3]) -> Self {
    Self {
        pre: FromField::from_field(input[0]),
        post: FromField::from_field(input[1]),
        block_of_change: FromField::from_field(input[2]),
    }
  }
}

impl<T> Eq for ScheduledValueChange<T> where T: Eq {
    fn eq(self, other: Self) -> bool {
        (self.pre == other.pre) & (self.post == other.post) & (self.block_of_change == other.block_of_change)
    }
}
