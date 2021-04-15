use noir_field::Bn254Scalar;

use crate::Backend;

mod proof_system;
pub mod pwg;
mod smart_contract;
pub struct Marlin;

impl Backend<Bn254Scalar> for Marlin {}
