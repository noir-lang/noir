use std::collections::HashMap;

use acir::BlackBoxFunc;

use num_bigint::BigUint;

use crate::BlackBoxResolutionError;

/// Resolve BigInt opcodes by storing BigInt values (and their moduli) by their ID in a HashMap:
/// - When it encounters a bigint operation opcode, it performs the operation on the stored values
/// and store the result using the provided ID.
/// - When it gets a to_bytes opcode, it simply looks up the value and resolves the output witness accordingly.
#[derive(Default, Debug, Clone, PartialEq, Eq)]

pub struct BigIntSolver {
    bigint_id_to_value: HashMap<u32, BigUint>,
    bigint_id_to_modulus: HashMap<u32, BigUint>,
}

impl BigIntSolver {
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
                lhs * rhs.modpow(&(&modulus - BigUint::from(2_u32)), &modulus)
            } //TODO ensure that modulus is prime
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
}

/// Wrapper over the generic bigint solver to automatically assign bigint IDs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BigIntSolverWithId {
    solver: BigIntSolver,
    last_id: u32,
}

impl BigIntSolverWithId {
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
