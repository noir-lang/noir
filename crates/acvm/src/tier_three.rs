// Listed below are backends with tier two support

pub use super::backends::csat_3_plonk_aztec::Plonk as CSAT_3_PLONK_AZTEC;
pub use super::backends::r1cs_marlin_arkworks::Marlin as R1CS_MARLIN_ARKWORKS;

use crate::Backend;

#[derive(Debug, Copy, Clone)]
pub enum TierThree {
    Csat3PlonkAztec,
    R1CSMarlinArkworks,
}

impl TierThree {
    pub(crate) fn fetch_backend(&self) -> Box<dyn Backend> {
        match self {
            TierThree::Csat3PlonkAztec => Box::new(CSAT_3_PLONK_AZTEC),
            TierThree::R1CSMarlinArkworks => Box::new(R1CS_MARLIN_ARKWORKS),
        }
    }
}

pub const TIER_THREE_MAP: [(&str, TierThree); 2] = [
    ("csat_3_plonk_aztec", TierThree::Csat3PlonkAztec),
    ("r1cs_marlin_arkworks", TierThree::R1CSMarlinArkworks),
];
