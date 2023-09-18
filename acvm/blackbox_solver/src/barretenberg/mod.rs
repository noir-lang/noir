use acir::{BlackBoxFunc, FieldElement};

use crate::{BlackBoxFunctionSolver, BlackBoxResolutionError};

mod wasm;

use wasm::Barretenberg;

use self::wasm::{Pedersen, ScalarMul, SchnorrSig};

#[deprecated = "The `BarretenbergSolver` is a temporary solution and will be removed in future."]
pub struct BarretenbergSolver {
    blackbox_vendor: Barretenberg,
}

#[allow(deprecated)]
impl BarretenbergSolver {
    #[cfg(target_arch = "wasm32")]
    pub async fn initialize() -> BarretenbergSolver {
        let blackbox_vendor = Barretenberg::initialize().await;
        BarretenbergSolver { blackbox_vendor }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> BarretenbergSolver {
        let blackbox_vendor = Barretenberg::new();
        BarretenbergSolver { blackbox_vendor }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(deprecated)]
impl Default for BarretenbergSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
impl BlackBoxFunctionSolver for BarretenbergSolver {
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

        #[allow(deprecated)]
        self.blackbox_vendor.verify_signature(pub_key, sig_s, sig_e, message).map_err(|err| {
            BlackBoxResolutionError::Failed(BlackBoxFunc::SchnorrVerify, err.to_string())
        })
    }

    fn pedersen(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        #[allow(deprecated)]
        self.blackbox_vendor
            .encrypt(inputs.to_vec(), domain_separator)
            .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::Pedersen, err.to_string()))
    }

    fn fixed_base_scalar_mul(
        &self,
        low: &FieldElement,
        high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        #[allow(deprecated)]
        self.blackbox_vendor.fixed_base(low, high).map_err(|err| {
            BlackBoxResolutionError::Failed(BlackBoxFunc::FixedBaseScalarMul, err.to_string())
        })
    }
}
