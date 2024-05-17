#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use acir::{BlackBoxFunc, FieldElement};
use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};

mod embedded_curve_ops;
mod poseidon2;
mod wasm;

pub use embedded_curve_ops::{embedded_curve_add, multi_scalar_mul};
pub use poseidon2::poseidon2_permutation;
use wasm::Barretenberg;

use self::wasm::{Pedersen, SchnorrSig};

pub struct Bn254BlackBoxSolver {
    blackbox_vendor: Barretenberg,
}

impl Bn254BlackBoxSolver {
    pub async fn initialize() -> Bn254BlackBoxSolver {
        // We fallback to the sync initialization of barretenberg on non-wasm targets.
        // This ensures that wasm packages consuming this still build on the default target (useful for linting, etc.)
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let blackbox_vendor = Barretenberg::initialize().await;
                Bn254BlackBoxSolver { blackbox_vendor }
            } else {
                Bn254BlackBoxSolver::new()
            }
        }
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
        signature: &[u8; 64],
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

    fn multi_scalar_mul(
        &self,
        points: &[FieldElement],
        scalars_lo: &[FieldElement],
        scalars_hi: &[FieldElement],
    ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
        multi_scalar_mul(points, scalars_lo, scalars_hi)
    }

    fn ec_add(
        &self,
        input1_x: &FieldElement,
        input1_y: &FieldElement,
        input1_infinite: &FieldElement,
        input2_x: &FieldElement,
        input2_y: &FieldElement,
        input2_infinite: &FieldElement,
    ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
        embedded_curve_add(
            [*input1_x, *input1_y, *input1_infinite],
            [*input2_x, *input2_y, *input2_infinite],
        )
    }

    fn poseidon2_permutation(
        &self,
        inputs: &[FieldElement],
        len: u32,
    ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
        poseidon2_permutation(inputs, len)
    }
}
