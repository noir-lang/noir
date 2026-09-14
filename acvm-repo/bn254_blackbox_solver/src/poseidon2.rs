use acir::AcirField;
use acvm_blackbox_solver::BlackBoxResolutionError;
use itertools::Itertools;

use crate::FieldElement;
use crate::poseidon2_constants::{POSEIDON2_CONFIG, Poseidon2Config};

pub fn poseidon2_permutation(
    inputs: &[FieldElement],
) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
    let poseidon = Poseidon2::new();
    poseidon.permutation(inputs)
}

pub fn poseidon2_config_state_size() -> u32 {
    POSEIDON2_CONFIG.t
}

struct Poseidon2<'a> {
    config: &'a Poseidon2Config,
}

impl Poseidon2<'_> {
    fn new() -> Self {
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
            state.iter_mut().zip_eq(self.config.round_constant[round])
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

#[cfg(test)]
mod tests {
    use super::poseidon2_permutation;
    use crate::poseidon2_constants::field_from_hex;

    /// Cross-check vectors against an independent Poseidon2 implementation.
    ///
    /// The expected outputs were generated with the `zkhash` crate v0.2.0
    /// (<https://github.com/HorizenLabs/poseidon2>), instantiating its
    /// `Poseidon2` permutation with the parameters from
    /// `crate::poseidon2_constants::POSEIDON2_CONFIG` (t = 4, d = 5) and
    /// feeding it the inputs below.
    #[test]
    fn matches_external_reference_implementation() {
        let cases: [([&str; 4], [&str; 4]); 5] = [
            // all zeroes
            (
                [
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000000",
                ],
                [
                    "18DFB8DC9B82229CFF974EFEFC8DF78B1CE96D9D844236B496785C698BC6732E",
                    "095C230D1D37A246E8D2D5A63B165FE0FADE040D442F61E25F0590E5FB76F839",
                    "0BB9545846E1AFA4FA3C97414A60A20FC4949F537A68CCECA34C5CE71E28AA59",
                    "18A4F34C9C6F99335FF7638B82AEED9018026618358873C982BBDDE265B2ED6D",
                ],
            ),
            // sequential small values
            (
                [
                    "0000000000000000000000000000000000000000000000000000000000000000",
                    "0000000000000000000000000000000000000000000000000000000000000001",
                    "0000000000000000000000000000000000000000000000000000000000000002",
                    "0000000000000000000000000000000000000000000000000000000000000003",
                ],
                [
                    "01BD538C2EE014ED5141B29E9AE240BF8DB3FE5B9A38629A9647CF8D76C01737",
                    "239B62E7DB98AA3A2A8F6A0D2FA1709E7A35959AA6C7034814D9DAA90CBAC662",
                    "04CBB44C61D928ED06808456BF758CBF0C18D1E15A7B6DBC8245FA7515D5E3CB",
                    "2E11C5CFF2A22C64D01304B778D78F6998EFF1AB73163A35603F54794C30847A",
                ],
            ),
            // u128::MAX in every lane
            (
                [
                    "00000000000000000000000000000000ffffffffffffffffffffffffffffffff",
                    "00000000000000000000000000000000ffffffffffffffffffffffffffffffff",
                    "00000000000000000000000000000000ffffffffffffffffffffffffffffffff",
                    "00000000000000000000000000000000ffffffffffffffffffffffffffffffff",
                ],
                [
                    "1452D1D69A606FB2F6AFF10FA4C73EA7486AC4BD59B3557B52311EFFB283A261",
                    "2433004A0EDE6798EF76B637F9E2A0EAB454D70B7433A9AB18512D5A980890A9",
                    "05A2ECD90756DD7DBD1840B0F252E490A73594CD103B56F6A3B1ADD8F38449BE",
                    "1D5B91141464C8B36F830F33B7BA06BEA37D309A7A5E63A91100BB23C55168F1",
                ],
            ),
            // p - 1 (largest field element) in every lane
            (
                [
                    "30644E72E131A029B85045B68181585D2833E84879B9709143E1F593F0000000",
                    "30644E72E131A029B85045B68181585D2833E84879B9709143E1F593F0000000",
                    "30644E72E131A029B85045B68181585D2833E84879B9709143E1F593F0000000",
                    "30644E72E131A029B85045B68181585D2833E84879B9709143E1F593F0000000",
                ],
                [
                    "1B18E6CA21A1E9B15D65F0B5861EDE5FF20DB8FA3722531823D0C817D69D945D",
                    "0AFB50EA6867B1CB2D9D1EAC935AF746BC7A780E181A1E6AE9B768C9CBA68878",
                    "0A521A22CA614E65B877D0676652FB60E90A11B462F9846A08E811D95272A9D8",
                    "2369F077784E0AEA99EE3DC6B7B01612AF7F80D7F08B755F9F116E2885EE367F",
                ],
            ),
            // arbitrary full-width distinct values
            (
                [
                    "123456789ABCDEF00FEDCBA987654321123456789ABCDEF00FEDCBA987654321",
                    "2718281828459045235360287471352662497757247093699959574966967627",
                    "1414213562373095048801688724209698078569671875376948073176679737",
                    "0B172182839274F8E5D4C3B2A1908070605040302010FFEEDDCCBBAA99887766",
                ],
                [
                    "1C68B20A2080BCC11A2B6F38A46F8270C3CE1DCD40CF8A16626E1CC936E90D56",
                    "22FDAD6F2E2AED646BE444EFB2AE2EAACD49F0440C846F4882F8B013C01C792C",
                    "1726C0B52C59E7008DBB710A8D3046214257D997A6E7870F46E7DFE5C6729378",
                    "03B4A4B3B3694B4EFAF50186E75062F30D3FF4B73D95CAB554763B7E19DE8BA0",
                ],
            ),
        ];

        for (inputs, expected) in cases {
            let inputs = inputs.map(field_from_hex);
            let expected = expected.map(field_from_hex);
            let result = poseidon2_permutation(&inputs).expect("should successfully permute");
            assert_eq!(result, expected, "mismatch for inputs {inputs:?}");
        }
    }
}
