use crate::Backend;

// Listed below are backends with tier two support

pub enum TierTwo {}

pub const TIER_TWO_MAP: [(&'static str, TierTwo); 0] = [];

impl TierTwo {
    pub(crate) fn fetch_backend(&self) -> Box<dyn Backend> {
        unreachable!("There are currently no backends in tier two")
    }
}
