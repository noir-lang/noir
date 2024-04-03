use std::collections::HashMap;

use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};

use num_bigint::BigUint;

use crate::pwg::OpcodeResolutionError;

/// Resolve BigInt opcodes by storing BigInt values (and their moduli) by their ID in a HashMap:
/// - When it encounters a bigint operation opcode, it performs the operation on the stored values
/// and store the result using the provided ID.
/// - When it gets a to_bytes opcode, it simply looks up the value and resolves the output witness accordingly.
#[derive(Default)]
pub(crate) struct BigIntSolver {
    bigint_id_to_value: HashMap<u32, BigUint>,
    bigint_id_to_modulus: HashMap<u32, BigUint>,
}

impl BigIntSolver {
    pub(crate) fn get_bigint(
        &self,
        id: u32,
        func: BlackBoxFunc,
    ) -> Result<BigUint, OpcodeResolutionError> {
        self.bigint_id_to_value
            .get(&id)
            .ok_or(OpcodeResolutionError::BlackBoxFunctionFailed(
                func,
                format!("could not find bigint of id {id}"),
            ))
            .cloned()
    }

    pub(crate) fn get_modulus(
        &self,
        id: u32,
        func: BlackBoxFunc,
    ) -> Result<BigUint, OpcodeResolutionError> {
        self.bigint_id_to_modulus
            .get(&id)
            .ok_or(OpcodeResolutionError::BlackBoxFunctionFailed(
                func,
                format!("could not find bigint of id {id}"),
            ))
            .cloned()
    }
    pub(crate) fn bigint_from_bytes(
        &mut self,
        inputs: &[FunctionInput],
        modulus: &[u8],
        output: u32,
        initial_witness: &mut WitnessMap,
    ) -> Result<(), OpcodeResolutionError> {
        let bytes = inputs
            .iter()
            .map(|input| initial_witness.get(&input.witness).unwrap().to_u128() as u8)
            .collect::<Vec<u8>>();
        let bigint = BigUint::from_bytes_le(&bytes);
        self.bigint_id_to_value.insert(output, bigint);
        let modulus = BigUint::from_bytes_le(modulus);
        self.bigint_id_to_modulus.insert(output, modulus);
        Ok(())
    }

    pub(crate) fn bigint_to_bytes(
        &self,
        input: u32,
        outputs: &[Witness],
        initial_witness: &mut WitnessMap,
    ) -> Result<(), OpcodeResolutionError> {
        let bigint = self.get_bigint(input, BlackBoxFunc::BigIntToLeBytes)?;

        let mut bytes = bigint.to_bytes_le();
        while bytes.len() < outputs.len() {
            bytes.push(0);
        }
        bytes.iter().zip(outputs.iter()).for_each(|(byte, output)| {
            initial_witness.insert(*output, FieldElement::from(*byte as u128));
        });
        Ok(())
    }

    pub(crate) fn bigint_op(
        &mut self,
        lhs: u32,
        rhs: u32,
        output: u32,
        func: BlackBoxFunc,
    ) -> Result<(), OpcodeResolutionError> {
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
                lhs * rhs.modpow(&(&modulus - BigUint::from(1_u32)), &modulus)
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
