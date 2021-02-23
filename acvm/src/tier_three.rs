// Listed below are backends with tier two support

pub use super::backends::csat_3_plonk_aztec::Plonk as CSAT_3_PLONK_AZTEC;

use crate::Backend;
use enum_iterator::IntoEnumIterator;

#[derive(Debug, IntoEnumIterator)]
pub enum TierThree {
    Csat3PlonkAztec,
}

impl TierThree {
    pub(crate) fn fetch_backend(&self) -> Box<dyn Backend> {
        match self {
            TierThree::Csat3PlonkAztec => Box::new(CSAT_3_PLONK_AZTEC),
        }
    }
}

pub const TIER_THREE_MAP: [(&'static str, TierThree); 1] =
    [("csat_3_plonk_aztec", TierThree::Csat3PlonkAztec)];
