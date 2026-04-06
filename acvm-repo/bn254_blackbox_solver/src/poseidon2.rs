use acir::AcirField;
use acvm_blackbox_solver::BlackBoxResolutionError;
use itertools::Itertools;

use crate::FieldElement;
use crate::poseidon2_constants::{POSEIDON2_CONFIG, Poseidon2Config};

/// Poseidon2 permutation dispatching on input length.
/// Currently supports T=4; the framework is ready for T=8 and T=12 when
/// constants and matrix operations are added.
pub fn poseidon2_permutation(
    inputs: &[FieldElement],
) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
    match inputs.len() {
        4 => Poseidon2::<4>::new().permutation(inputs),
        len => Err(BlackBoxResolutionError::Failed(
            acir::BlackBoxFunc::Poseidon2Permutation,
            format!("Unsupported Poseidon2 state size: {len}. Supported values: 4"),
        )),
    }
}

pub fn poseidon2_config_state_size() -> u32 {
    4
}

struct Poseidon2<const T: usize> {
    config: &'static Poseidon2Config<T>,
}

impl Poseidon2<4> {
    fn new() -> Self {
        Poseidon2 { config: &POSEIDON2_CONFIG }
    }
}

impl<const T: usize> Poseidon2<T> {
    fn single_box(x: FieldElement) -> FieldElement {
        let s = x * x;
        s * s * x
    }

    fn s_box(input: &mut [FieldElement; T]) {
        for i in input {
            *i = Self::single_box(*i);
        }
    }

    fn add_round_constants(&self, state: &mut [FieldElement; T], round: usize) {
        for (state_element, constant_element) in
            state.iter_mut().zip_eq(self.config.round_constant[round])
        {
            *state_element += constant_element;
        }
    }

    /// Dispatches to the appropriate matrix multiplication for the state width T.
    fn external_matrix_multiplication(input: &mut [FieldElement; T]) {
        match T {
            4 => Self::matrix_multiplication_4x4(input.as_mut_slice()),
            _ => unimplemented!("Matrix multiplication for T={T} not yet implemented"),
        }
    }

    /// Algorithm is taken directly from the Poseidon2 implementation in Barretenberg crypto module.
    fn matrix_multiplication_4x4(input: &mut [FieldElement]) {
        assert!(input.len() == 4);
        let t0 = input[0] + input[1]; // A + B
        let t1 = input[2] + input[3]; // C + D
        let mut t2 = input[1] + input[1]; // 2B
        t2 += t1; // 2B + C + D
        let mut t3 = input[3] + input[3]; // 2D
        t3 += t0; // 2D + A + B
        let mut t4 = t1 + t1;
        t4 += t4;
        t4 += t3; // A + B + 4C + 6D
        let mut t5 = t0 + t0;
        t5 += t5;
        t5 += t2; // 4A + 6B + C + D
        let t6 = t3 + t5; // 5A + 7B + C + 3D
        let t7 = t2 + t4; // A + 3B + 5C + 7D
        input[0] = t6;
        input[1] = t5;
        input[2] = t7;
        input[3] = t4;
    }

    fn internal_m_multiplication(&self, input: &mut [FieldElement; T]) {
        let mut sum = FieldElement::zero();
        for i in input.iter() {
            sum += *i;
        }
        for (index, i) in input.iter_mut().enumerate() {
            *i = *i * self.config.internal_matrix_diagonal[index];
            *i += sum;
        }
    }

    pub(crate) fn permutation(
        &self,
        inputs: &[FieldElement],
    ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
        if inputs.len() != T {
            return Err(BlackBoxResolutionError::Failed(
                acir::BlackBoxFunc::Poseidon2Permutation,
                format!("Expected {T} values but encountered {}", inputs.len()),
            ));
        }

        let mut state = [FieldElement::zero(); T];
        for (index, input) in inputs.iter().enumerate() {
            state[index] = *input;
        }

        // Apply 1st linear layer
        Self::external_matrix_multiplication(&mut state);

        // First set of external rounds
        let rf_first = self.config.rounds_f / 2;
        for r in 0..rf_first {
            self.add_round_constants(&mut state, r as usize);
            Self::s_box(&mut state);
            Self::external_matrix_multiplication(&mut state);
        }

        // Internal rounds
        let p_end = rf_first + self.config.rounds_p;
        for r in rf_first..p_end {
            state[0] += self.config.round_constant[r as usize][0];
            state[0] = Self::single_box(state[0]);
            self.internal_m_multiplication(&mut state);
        }

        // Remaining external rounds
        let num_rounds = self.config.rounds_f + self.config.rounds_p;
        for i in p_end..num_rounds {
            self.add_round_constants(&mut state, i as usize);
            Self::s_box(&mut state);
            Self::external_matrix_multiplication(&mut state);
        }

        Ok(state.into())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use acir::AcirField;

    use proptest::prelude::*;
    use proptest::result::maybe_ok;

    use super::{FieldElement, poseidon2_permutation};
    use crate::poseidon2_constants::{POSEIDON2_CONFIG, field_from_hex};

    #[test]
    fn smoke_test() {
        let inputs = [FieldElement::zero(); 4];
        let result = poseidon2_permutation(&inputs).expect("should successfully permute");

        let expected_result = [
            field_from_hex("18DFB8DC9B82229CFF974EFEFC8DF78B1CE96D9D844236B496785C698BC6732E"),
            field_from_hex("095C230D1D37A246E8D2D5A63B165FE0FADE040D442F61E25F0590E5FB76F839"),
            field_from_hex("0BB9545846E1AFA4FA3C97414A60A20FC4949F537A68CCECA34C5CE71E28AA59"),
            field_from_hex("18A4F34C9C6F99335FF7638B82AEED9018026618358873C982BBDDE265B2ED6D"),
        ];
        assert_eq!(result, expected_result);
    }

    fn into_old_ark_field<T, U>(field: T) -> U
    where
        T: AcirField,
        U: ark_ff_v04::PrimeField,
    {
        U::from_be_bytes_mod_order(&field.to_be_bytes())
    }

    fn into_new_ark_field<T, U>(field: T) -> U
    where
        T: ark_ff_v04::PrimeField,
        U: ark_ff::PrimeField,
    {
        use zkhash::ark_ff::BigInteger;

        U::from_be_bytes_mod_order(&field.into_bigint().to_bytes_be())
    }

    fn run_both_poseidon2_permutations(
        inputs: Vec<FieldElement>,
    ) -> (Vec<ark_bn254::Fr>, Vec<ark_bn254::Fr>) {
        let poseidon2_t = 4usize;
        let poseidon2_d = 5;
        let rounds_f = POSEIDON2_CONFIG.rounds_f as usize;
        let rounds_p = POSEIDON2_CONFIG.rounds_p as usize;
        let mat_internal_diag_m_1: Vec<ark_bn254_v04::Fr> =
            POSEIDON2_CONFIG.internal_matrix_diagonal.into_iter().map(into_old_ark_field).collect();
        let mat_internal = vec![];
        let round_constants: Vec<Vec<ark_bn254_v04::Fr>> = POSEIDON2_CONFIG
            .round_constant
            .iter()
            .map(|fields| fields.iter().copied().map(into_old_ark_field).collect())
            .collect();

        let external_poseidon2 = zkhash::poseidon2::poseidon2::Poseidon2::new(&Arc::new(
            zkhash::poseidon2::poseidon2_params::Poseidon2Params::new(
                poseidon2_t,
                poseidon2_d,
                rounds_f,
                rounds_p,
                &mat_internal_diag_m_1,
                &mat_internal,
                &round_constants,
            ),
        ));

        let result =
            poseidon2_permutation(&inputs).unwrap().into_iter().map(|x| x.into_repr()).collect();

        let expected_result = external_poseidon2.permutation(
            &inputs.into_iter().map(into_old_ark_field).collect::<Vec<ark_bn254_v04::Fr>>(),
        );
        (result, expected_result.into_iter().map(into_new_ark_field).collect())
    }

    prop_compose! {
        // Use both `u128` and hex proptest strategies
        fn field_element()
            (u128_or_hex in maybe_ok(any::<u128>(), "[0-9a-f]{64}"))
            -> FieldElement
        {
            match u128_or_hex {
                Ok(number) => FieldElement::from(number),
                Err(hex) => FieldElement::from_hex(&hex).expect("should accept any 32 byte hex string"),
            }
        }
    }

    proptest! {
        #[test]
        fn poseidon2_permutation_matches_external_impl(inputs in proptest::collection::vec(field_element(), 4)) {
            let (result, expected_result) = run_both_poseidon2_permutations(inputs);
            prop_assert_eq!(result, expected_result);
        }
    }
}
