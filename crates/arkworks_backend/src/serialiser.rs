use crate::bridge::Bn254Acir;
use acir::circuit::Circuit;
use ark_bn254::Fr;

/// Converts an ACIR into an ACIR struct that
/// the arkworks backend can consume
pub fn serialise(acir: Circuit, values: Vec<Fr>) -> Bn254Acir {
    (acir, values).into()
}
