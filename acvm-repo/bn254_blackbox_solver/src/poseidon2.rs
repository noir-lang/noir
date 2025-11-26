use acir::AcirField;
use acvm_blackbox_solver::BlackBoxResolutionError;

use crate::FieldElement;
use crate::poseidon2_constants::{POSEIDON2_CONFIG, Poseidon2Config};

pub fn poseidon2_permutation(
    inputs: &[FieldElement],
) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
    let poseidon = Poseidon2::new();
    poseidon.permutation(inputs)
}

pub(crate) struct Poseidon2<'a> {
    config: &'a Poseidon2Config,
}

impl Poseidon2<'_> {
    pub(crate) fn new() -> Self {
        Poseidon2 { config: &POSEIDON2_CONFIG }
    }

    fn single_box(x: FieldElement) -> FieldElement {
        let s = x * x;
        s * s * x
    }

    fn s_box(input: &mut [FieldElement]) {
        for i in input {
            *i = Self::single_box(*i);
        }
    }

    fn add_round_constants(&self, state: &mut [FieldElement], round: usize) {
        for (state_element, constant_element) in
            state.iter_mut().zip(self.config.round_constant[round])
        {
            *state_element += constant_element;
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

    fn internal_m_multiplication(&self, input: &mut [FieldElement]) {
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
        if inputs.len() != self.config.t as usize {
            return Err(BlackBoxResolutionError::Failed(
                acir::BlackBoxFunc::Poseidon2Permutation,
                format!("Expected {} values but encountered {}", self.config.t, inputs.len()),
            ));
        }
        // Read witness assignments
        let mut state = [FieldElement::zero(); 4];
        for (index, input) in inputs.iter().enumerate() {
            state[index] = *input;
        }
        // Apply 1st linear layer
        Self::matrix_multiplication_4x4(&mut state);

        // First set of external rounds
        let rf_first = self.config.rounds_f / 2;
        for r in 0..rf_first {
            self.add_round_constants(&mut state, r as usize);
            Self::s_box(&mut state);
            Self::matrix_multiplication_4x4(&mut state);
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
            Self::matrix_multiplication_4x4(&mut state);
        }
        Ok(state.into())
    }
}

/// Performs a poseidon hash with a sponge construction equivalent to the one in the Barretenberg proving system
pub fn poseidon_hash(inputs: &[FieldElement]) -> Result<FieldElement, BlackBoxResolutionError> {
    let two_pow_64 = 18446744073709551616_u128.into();
    let iv = FieldElement::from(inputs.len()) * two_pow_64;
    // A rate of 3, with a width of 4, gives 1 field element of capacity, i.e ~128 bits of security
    let mut sponge = Poseidon2Sponge::new(iv, 3);
    for input in inputs.iter() {
        sponge.absorb(*input)?;
    }
    sponge.squeeze()
}

pub struct Poseidon2Sponge<'a> {
    rate: usize,
    poseidon: Poseidon2<'a>,
    squeezed: bool,
    cache: Vec<FieldElement>,
    state: Vec<FieldElement>,
}

impl<'a> Poseidon2Sponge<'a> {
    pub fn new(iv: FieldElement, rate: usize) -> Poseidon2Sponge<'a> {
        let mut result = Poseidon2Sponge {
            cache: Vec::with_capacity(rate),
            state: vec![FieldElement::zero(); rate + 1],
            squeezed: false,
            rate,
            poseidon: Poseidon2::new(),
        };
        result.state[rate] = iv;
        result
    }

    fn perform_duplex(&mut self) -> Result<(), BlackBoxResolutionError> {
        // zero-pad the cache
        for _ in self.cache.len()..self.rate {
            self.cache.push(FieldElement::zero());
        }
        // add the cache into sponge state
        for i in 0..self.rate {
            self.state[i] += self.cache[i];
        }
        self.state = self.poseidon.permutation(&self.state)?;
        Ok(())
    }

    pub fn absorb(&mut self, input: FieldElement) -> Result<(), BlackBoxResolutionError> {
        assert!(!self.squeezed);
        if self.cache.len() == self.rate {
            // If we're absorbing, and the cache is full, apply the sponge permutation to compress the cache
            self.perform_duplex()?;
            self.cache = vec![input];
        } else {
            // If we're absorbing, and the cache is not full, add the input into the cache
            self.cache.push(input);
        }
        Ok(())
    }

    pub fn squeeze(&mut self) -> Result<FieldElement, BlackBoxResolutionError> {
        assert!(!self.squeezed);
        // If we're in absorb mode, apply sponge permutation to compress the cache.
        self.perform_duplex()?;
        self.squeezed = true;

        // Pop one item off the top of the permutation and return it.
        Ok(self.state[0])
    }
}

#[cfg(test)]
mod test {
    use acir::AcirField;

    use super::{FieldElement, poseidon2_permutation};
    use crate::poseidon2_constants::field_from_hex;

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

    #[test]
    fn hash_smoke_test() {
        let fields = [
            FieldElement::from(1u128),
            FieldElement::from(2u128),
            FieldElement::from(3u128),
            FieldElement::from(4u128),
        ];
        let result = super::poseidon_hash(&fields).expect("should hash successfully");
        assert_eq!(
            result,
            field_from_hex("130bf204a32cac1f0ace56c78b731aa3809f06df2731ebcf6b3464a15788b1b9"),
        );
    }
}
