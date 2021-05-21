use crate::bridge::{AcirArithGate, AcirCircuit};
use noir_field::FieldElement;

pub type CurveAcir = AcirCircuit<Fr>;
pub type CurveAcirArithGate = AcirArithGate<Fr>;

cfg_if::cfg_if! {
    if #[cfg(feature = "bn254")] {
        pub use ark_bn254::{Bn254 as Curve, Fr};

        // Converts a FieldElement to a Fr
        // noir_field uses arkworks for bn254
        pub fn from_fe(fe : FieldElement) -> Fr {
            fe.into_repr()
        }
    } else if #[cfg(feature = "bls12_381")] {
        pub use ark_bls12_381::{Bls12_381 as Curve, Fr};

        // Converts a FieldElement to a Fr
        // noir_field uses arkworks for bls12_381
        pub fn from_fe(fe : FieldElement) -> Fr {
            fe.into_repr()
        }
    } else {
        compile_error!("please specify a field to compile with");
    }
}
