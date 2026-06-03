#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

mod poseidon2;
mod poseidon2_constants;

use acir::{AcirField, BlackBoxFunc};
use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use ark_ec::{AffineRepr, CurveGroup, VariableBaseMSM};
use ark_ff::{BigInt, MontConfig, PrimeField, Zero};

type FieldElement = acir::acir_field::GenericFieldElement<ark_tom256::Fq>;

#[derive(Default)]
pub struct T256BlackboxSolver;

impl T256BlackboxSolver {
    fn coordinates_to_affine(
        x: FieldElement,
        y: FieldElement,
    ) -> Result<ark_secp256r1::Affine, BlackBoxResolutionError> {
        Ok(ark_secp256r1::Affine::new_unchecked(
            ark_secp256r1::Fq::from_bigint(x.into_repr().into_bigint()).unwrap(),
            ark_secp256r1::Fq::from_bigint(y.into_repr().into_bigint()).unwrap(),
        ))
    }

    // Taken from the embedded_curve_ops in the Bn254 blackbox solver
    fn parse_msm_inputs(
        points: &[FieldElement],
        scalars_lo: &[FieldElement],
        scalars_hi: &[FieldElement],
    ) -> Result<(Vec<ark_secp256r1::Affine>, Vec<BigInt<4>>), BlackBoxResolutionError> {
        if points.len() != 2 * scalars_lo.len() || scalars_lo.len() != scalars_hi.len() {
            return Err(BlackBoxResolutionError::Failed(
                BlackBoxFunc::MultiScalarMul,
                "Points and scalars must have the same length".to_string(),
            ));
        }

        // Collect all bases (affine points) and scalars for batch MSM
        let mut bases = Vec::new();
        let mut big_ints = Vec::new();

        for i in (0..points.len()).step_by(2) {
            let point = Self::coordinates_to_affine(points[i], points[i + 1]).map_err(|e| {
                BlackBoxResolutionError::Failed(BlackBoxFunc::MultiScalarMul, e.to_string())
            })?;

            let scalar_low: u128 = T256BlackboxSolver::field_to_u128_limb(
                &scalars_lo[i / 2],
                BlackBoxFunc::MultiScalarMul,
            )?;

            let scalar_high: u128 = T256BlackboxSolver::field_to_u128_limb(
                &scalars_hi[i / 2],
                BlackBoxFunc::MultiScalarMul,
            )?;

            // Convert to BigInt<4>, using u64 limbs.
            let limbs_array = [
                scalar_low as u64,
                (scalar_low >> 64) as u64,
                scalar_high as u64,
                (scalar_high >> 64) as u64,
            ];
            let scalar_bigint = BigInt::new(limbs_array);

            // Check if this is smaller than the P256 modulus
            if scalar_bigint >= ark_secp256r1::FrConfig::MODULUS {
                // Format as hex string (big-endian, most significant limb first)
                let hex_str = format!(
                    "{:016x}{:016x}{:016x}{:016x}",
                    limbs_array[3], limbs_array[2], limbs_array[1], limbs_array[0]
                );
                return Err(BlackBoxResolutionError::Failed(
                    BlackBoxFunc::MultiScalarMul,
                    format!("{hex_str} is not a valid T256 scalar"),
                ));
            }

            bases.push(point);
            big_ints.push(scalar_bigint);
        }
        Ok((bases, big_ints))
    }

    fn field_to_u128_limb(
        limb: &FieldElement,
        func: BlackBoxFunc,
    ) -> Result<u128, BlackBoxResolutionError> {
        limb.try_into_u128().ok_or_else(|| {
            BlackBoxResolutionError::Failed(
                func,
                format!("Limb {} is not less than 2^128", limb.to_hex()),
            )
        })
    }
}

impl BlackBoxFunctionSolver<FieldElement> for T256BlackboxSolver {
    fn multi_scalar_mul(
        &self,
        points: &[FieldElement],
        scalars_lo: &[FieldElement],
        scalars_hi: &[FieldElement],
        predicate: bool,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        if !predicate {
            return Ok((FieldElement::zero(), FieldElement::zero()));
        }

        let (points, scalars) = Self::parse_msm_inputs(points, scalars_lo, scalars_hi)?;

        let msm_result =
            ark_secp256r1::Affine::from(ark_secp256r1::Projective::msm_bigint(&points, &scalars));

        if let Some((out_x, out_y)) = msm_result.xy() {
            Ok((
                FieldElement::from_repr(ark_tom256::Fq::from_bigint(out_x.into_bigint()).unwrap()),
                FieldElement::from_repr(ark_tom256::Fq::from_bigint(out_y.into_bigint()).unwrap()),
            ))
        } else {
            Ok((FieldElement::zero(), FieldElement::zero()))
        }
    }

    fn ec_add(
        &self,
        input1_x: &FieldElement,
        input1_y: &FieldElement,
        input2_x: &FieldElement,
        input2_y: &FieldElement,
        predicate: bool,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        if !predicate {
            return Ok((FieldElement::zero(), FieldElement::zero()));
        }

        let p1 = Self::coordinates_to_affine(*input1_x, *input1_y)?;
        let p2 = Self::coordinates_to_affine(*input2_x, *input2_y)?;

        let sum = ark_secp256r1::Affine::from(p1 + p2);
        if let Some((x, y)) = sum.xy() {
            Ok((
                FieldElement::from_repr(ark_tom256::Fq::from_bigint(x.into_bigint()).unwrap()),
                FieldElement::from_repr(ark_tom256::Fq::from_bigint(y.into_bigint()).unwrap()),
            ))
        } else {
            Ok((FieldElement::zero(), FieldElement::zero()))
        }
    }

    fn poseidon2_permutation(
        &self,
        inputs: &[FieldElement],
    ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
        Ok(
            // TODO: this is obviously wrong and just here for the purpose of the comptime code from
            //     noir stdlib to compile
            inputs.to_vec(), // poseidon2::poseidon2_permutation(inputs)?
        )
    }
}
