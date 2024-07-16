use dep::protocol_types::traits::{Serialize, Deserialize, FromField, ToField};
use std::cmp::min;

mod test;

// This data structure is used by SharedMutable to store the minimum delay with which a ScheduledValueChange object can
// schedule a change.
// This delay is initally equal to INITIAL_DELAY, and can be safely mutated to any other value over time. This mutation 
// is performed via `schedule_change` in order to satisfy ScheduleValueChange constraints: if e.g. we allowed for the 
// delay to be decreased immediately then it'd be possible for the state variable to schedule a value change with a 
// reduced delay, invalidating prior private reads.
struct ScheduledDelayChange<let INITIAL_DELAY: u32> {
    // Both pre and post are stored in public storage, so by default they are zeroed. By wrapping them in an Option, 
    // they default to Option::none(), which we detect and replace with INITIAL_DELAY. The end result is that a
    // ScheduledDelayChange that has not been initialized has a delay equal to INITIAL_DELAY, which is the desired
    // effect. Once initialized, the Option will never be none again.
    pre: Option<u32>,
    post: Option<u32>,
    // Block at which `post` value is used instead of `pre`
    block_of_change: u32,
}

impl<let INITIAL_DELAY: u32> ScheduledDelayChange<INITIAL_DELAY> {
    pub fn new(pre: Option<u32>, post: Option<u32>, block_of_change: u32) -> Self {
        Self { pre, post, block_of_change }
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
        }
    }
}

impl<INITIAL_DELAY> Eq for ScheduledDelayChange<INITIAL_DELAY> {
    fn eq(self, other: Self) -> bool {
        (self.pre == other.pre) & (self.post == other.post) & (self.block_of_change == other.block_of_change)
    }
}
