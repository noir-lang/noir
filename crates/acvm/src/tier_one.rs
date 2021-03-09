use crate::Backend;

// Listed below are backends with tier one support

#[derive(Debug, Copy, Clone)]
pub enum TierOne {}

pub const TIER_ONE_MAP: [(&'static str, TierOne); 0] = [];

impl TierOne {
    pub(crate) fn fetch_backend(&self) -> Box<dyn Backend> {
        unreachable!("There are currently no backends in tier one")
    }
}
