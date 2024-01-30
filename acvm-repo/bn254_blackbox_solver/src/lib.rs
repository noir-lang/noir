#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use acir::{BlackBoxFunc, FieldElement};
use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};

mod fixed_base_scalar_mul;
mod wasm;

pub use fixed_base_scalar_mul::{embedded_curve_add, fixed_base_scalar_mul};
use wasm::Barretenberg;

use self::wasm::{Pedersen, SchnorrSig};

pub struct Bn254BlackBoxSolver {
    blackbox_vendor: Barretenberg,
}

impl Bn254BlackBoxSolver {
    #[cfg(target_arch = "wasm32")]
    pub async fn initialize() -> Bn254BlackBoxSolver {
        let blackbox_vendor = Barretenberg::initialize().await;
        Bn254BlackBoxSolver { blackbox_vendor }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Bn254BlackBoxSolver {
        let blackbox_vendor = Barretenberg::new();
        Bn254BlackBoxSolver { blackbox_vendor }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for Bn254BlackBoxSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackBoxFunctionSolver for Bn254BlackBoxSolver {
    fn schnorr_verify(
        &self,
        public_key_x: &FieldElement,
        public_key_y: &FieldElement,
        signature: &[u8],
        message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError> {
        let pub_key_bytes: Vec<u8> =
            public_key_x.to_be_bytes().iter().copied().chain(public_key_y.to_be_bytes()).collect();

        let pub_key: [u8; 64] = pub_key_bytes.try_into().unwrap();
        let sig_s: [u8; 32] = signature[0..32].try_into().unwrap();
        let sig_e: [u8; 32] = signature[32..64].try_into().unwrap();

        self.blackbox_vendor.verify_signature(pub_key, sig_s, sig_e, message).map_err(|err| {
            BlackBoxResolutionError::Failed(BlackBoxFunc::SchnorrVerify, err.to_string())
        })
    }

    fn pedersen_commitment(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        #[allow(deprecated)]
        self.blackbox_vendor.encrypt(inputs.to_vec(), domain_separator).map_err(|err| {
            BlackBoxResolutionError::Failed(BlackBoxFunc::PedersenCommitment, err.to_string())
        })
    }

    fn pedersen_hash(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<FieldElement, BlackBoxResolutionError> {
        #[allow(deprecated)]
        self.blackbox_vendor.hash(inputs.to_vec(), domain_separator).map_err(|err| {
            BlackBoxResolutionError::Failed(BlackBoxFunc::PedersenCommitment, err.to_string())
        })
    }

    fn fixed_base_scalar_mul(
        &self,
        low: &FieldElement,
        high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        fixed_base_scalar_mul(low, high)
    }

    fn ec_add(
        &self,
        input1_x: &FieldElement,
        input1_y: &FieldElement,
        input2_x: &FieldElement,
        input2_y: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        embedded_curve_add(*input1_x, *input1_y, *input2_x, *input2_y)
    }
}
