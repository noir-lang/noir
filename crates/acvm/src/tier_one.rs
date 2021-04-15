use noir_field::FieldElement;

use crate::Backend;

// Listed below are backends with tier one support

#[derive(Debug, Copy, Clone)]
pub enum TierOne {}

pub const TIER_ONE_MAP: [(&str, TierOne); 0] = [];

impl TierOne {
    pub(crate) fn fetch_backend<F: FieldElement>(&self) -> Box<dyn Backend<F>> {
        unreachable!("There are currently no backends in tier one")
    }
}
