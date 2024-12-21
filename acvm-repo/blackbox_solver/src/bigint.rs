use std::collections::HashMap;

use acir::BlackBoxFunc;

use num_bigint::BigUint;

use crate::BlackBoxResolutionError;

/// Resolve BigInt opcodes by storing BigInt values (and their moduli) by their ID in a HashMap:
/// - When it encounters a bigint operation opcode, it performs the operation on the stored values
///   and store the result using the provided ID.
/// - When it gets a to_bytes opcode, it simply looks up the value and resolves the output witness accordingly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigIntSolver {
    bigint_id_to_value: HashMap<u32, BigUint>,
    bigint_id_to_modulus: HashMap<u32, BigUint>,

    // Use pedantic ACVM solving
    pedantic_solving: bool,
}

impl BigIntSolver {
    pub fn with_pedantic_solving(pedantic_solving: bool) -> BigIntSolver {
        BigIntSolver {
            bigint_id_to_value: Default::default(),
            bigint_id_to_modulus: Default::default(),
            pedantic_solving,
        }
    }

    pub fn pedantic_solving(&self) -> bool {
        self.pedantic_solving
    }

    pub fn get_bigint(
        &self,
        id: u32,
        func: BlackBoxFunc,
    ) -> Result<BigUint, BlackBoxResolutionError> {
        self.bigint_id_to_value
            .get(&id)
            .ok_or(BlackBoxResolutionError::Failed(
                func,
                format!("could not find bigint of id {id}"),
            ))
            .cloned()
    }

    pub fn get_modulus(
        &self,
        id: u32,
        func: BlackBoxFunc,
    ) -> Result<BigUint, BlackBoxResolutionError> {
        self.bigint_id_to_modulus
            .get(&id)
            .ok_or(BlackBoxResolutionError::Failed(
                func,
                format!("could not find bigint of id {id}"),
            ))
            .cloned()
    }
    pub fn bigint_from_bytes(
        &mut self,
        inputs: &[u8],
        modulus: &[u8],
        output: u32,
    ) -> Result<(), BlackBoxResolutionError> {
        if self.pedantic_solving {
            if !self.is_valid_modulus(modulus) {
                panic!("--pedantic-solving: bigint_from_bytes: disallowed modulus {:?}", modulus);
            }
            if inputs.len() > modulus.len() {
                panic!(
                    "--pedantic-solving: bigint_from_bytes: inputs.len() > modulus.len() {:?}",
                    (inputs.len(), modulus.len())
                );
            }
        }
        let bigint = BigUint::from_bytes_le(inputs);
        self.bigint_id_to_value.insert(output, bigint);
        let modulus = BigUint::from_bytes_le(modulus);
        self.bigint_id_to_modulus.insert(output, modulus);
        Ok(())
    }

    pub fn bigint_to_bytes(&self, input: u32) -> Result<Vec<u8>, BlackBoxResolutionError> {
        let bigint = self.get_bigint(input, BlackBoxFunc::BigIntToLeBytes)?;
        Ok(bigint.to_bytes_le())
    }

    pub fn bigint_op(
        &mut self,
        lhs: u32,
        rhs: u32,
        output: u32,
        func: BlackBoxFunc,
    ) -> Result<(), BlackBoxResolutionError> {
        let modulus = self.get_modulus(lhs, func)?;
        let lhs = self.get_bigint(lhs, func)?;
        let rhs = self.get_bigint(rhs, func)?;
        let mut result = match func {
            BlackBoxFunc::BigIntAdd => lhs + rhs,
            BlackBoxFunc::BigIntSub => {
                if lhs >= rhs {
                    &lhs - &rhs
                } else {
                    &lhs + &modulus - &rhs
                }
            }
            BlackBoxFunc::BigIntMul => lhs * rhs,
            BlackBoxFunc::BigIntDiv => {
                if self.pedantic_solving && rhs == BigUint::ZERO {
                    return Err(BlackBoxResolutionError::Failed(
                        func,
                        "Attempted to divide BigInt by zero".to_string(),
                    ));
                }
                lhs * rhs.modpow(&(&modulus - BigUint::from(2_u32)), &modulus)
            }
            _ => unreachable!("ICE - bigint_op must be called for an operation"),
        };
        if result > modulus {
            let q = &result / &modulus;
            result -= q * &modulus;
        }
        self.bigint_id_to_value.insert(output, result);
        self.bigint_id_to_modulus.insert(output, modulus);
        Ok(())
    }

    pub fn allowed_bigint_moduli() -> Vec<Vec<u8>> {
        let bn254_fq: Vec<u8> = vec![
            0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a,
            0x81, 0x97, 0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1,
            0x72, 0x4e, 0x64, 0x30,
        ];
        let bn254_fr: Vec<u8> = vec![
            1, 0, 0, 240, 147, 245, 225, 67, 145, 112, 185, 121, 72, 232, 51, 40, 93, 88, 129, 129,
            182, 69, 80, 184, 41, 160, 49, 225, 114, 78, 100, 48,
        ];
        let secpk1_fr: Vec<u8> = vec![
            0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC,
            0xAE, 0xBA, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let secpk1_fq: Vec<u8> = vec![
            0x2F, 0xFC, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let secpr1_fq: Vec<u8> = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let secpr1_fr: Vec<u8> = vec![
            81, 37, 99, 252, 194, 202, 185, 243, 132, 158, 23, 167, 173, 250, 230, 188, 255, 255,
            255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255,
        ];
        vec![bn254_fq, bn254_fr, secpk1_fr, secpk1_fq, secpr1_fq, secpr1_fr]
    }

    pub fn is_valid_modulus(&self, modulus: &[u8]) -> bool {
        Self::allowed_bigint_moduli().into_iter().any(|allowed_modulus| allowed_modulus == modulus)
    }
}

/// Wrapper over the generic bigint solver to automatically assign bigint IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigIntSolverWithId {
    solver: BigIntSolver,
    last_id: u32,
}

impl BigIntSolverWithId {
    pub fn with_pedantic_solving(pedantic_solving: bool) -> BigIntSolverWithId {
        let solver = BigIntSolver::with_pedantic_solving(pedantic_solving);
        BigIntSolverWithId { solver, last_id: Default::default() }
    }

    pub fn create_bigint_id(&mut self) -> u32 {
        let output = self.last_id;
        self.last_id += 1;
        output
    }

    pub fn bigint_from_bytes(
        &mut self,
        inputs: &[u8],
        modulus: &[u8],
    ) -> Result<u32, BlackBoxResolutionError> {
        let id = self.create_bigint_id();
        self.solver.bigint_from_bytes(inputs, modulus, id)?;
        Ok(id)
    }

    pub fn bigint_to_bytes(&self, input: u32) -> Result<Vec<u8>, BlackBoxResolutionError> {
        self.solver.bigint_to_bytes(input)
    }

    pub fn bigint_op(
        &mut self,
        lhs: u32,
        rhs: u32,
        func: BlackBoxFunc,
    ) -> Result<u32, BlackBoxResolutionError> {
        let modulus_lhs = self.solver.get_modulus(lhs, func)?;
        let modulus_rhs = self.solver.get_modulus(rhs, func)?;
        if modulus_lhs != modulus_rhs {
            return Err(BlackBoxResolutionError::Failed(
                func,
                "moduli should be identical in BigInt operation".to_string(),
            ));
        }
        let id = self.create_bigint_id();
        self.solver.bigint_op(lhs, rhs, id, func)?;
        Ok(id)
    }
}

#[test]
fn all_allowed_bigint_moduli_are_prime() {
    use num_prime::nt_funcs::is_prime;
    use num_prime::Primality;

    for modulus in BigIntSolver::allowed_bigint_moduli() {
        let modulus = BigUint::from_bytes_le(&modulus);
        let prime_test_config = None;
        match is_prime(&modulus, prime_test_config) {
            Primality::Yes => (),
            Primality::No => panic!("not all allowed_bigint_moduli are prime: {modulus}"),
            Primality::Probable(probability) => {
                if probability < 0.90 {
                    panic!("not all allowed_bigint_moduli are prime within the allowed probability: {} < 0.90", probability);
                }
            }
        }
    }
}
