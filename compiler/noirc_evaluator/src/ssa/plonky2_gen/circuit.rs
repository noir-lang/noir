use super::config::{P2CircuitData, P2Field, P2ProofWithPublicInputs};
use super::noir_to_plonky2_field;
use crate::debug_trace::DebugTraceList;
use crate::errors::{Plonky2GenError, Plonky2VerifyError};
use acvm::acir::native_types::WitnessMap;
use acvm::FieldElement;
use noirc_abi::{input_parser::InputValue, InputMap};
use plonky2::iop::{
    target::Target,
    witness::{PartialWitness, WitnessWrite},
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Plonky2Circuit {
    pub data: P2CircuitData,
    pub parameters: Vec<Target>,
    pub parameter_names: Vec<String>,
    pub debug_trace_list: Option<DebugTraceList>,
}

impl Clone for Plonky2Circuit {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl std::hash::Hash for Plonky2Circuit {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        self.parameters.hash(state);
        self.parameter_names.hash(state);
        self.debug_trace_list.hash(state);
    }
}

impl Serialize for Plonky2Circuit {
    // see the todo at Clone
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Plonky2Circuit {
    // see the todo at Clone
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl Plonky2Circuit {
    pub fn prove(&self, inputs: &InputMap) -> Result<Vec<u8>, Plonky2GenError> {
        let mut pw = PartialWitness::new();
        let mut j = 0;
        for param_name in self.parameter_names.iter() {
            let value = inputs[param_name].clone();

            match value {
                InputValue::Field(field) => {
                    self.set_parameter(&mut j, field, &mut pw);
                }

                InputValue::Vec(input_values) => {
                    let _ = self.set_array_parameter(&mut j, &input_values, &mut pw)?;
                }

                InputValue::Struct(fields) => {
                    let mut input_values = Vec::new();
                    for value in fields.values() {
                        input_values.push(value.clone());
                    }
                    let _ = self.set_array_parameter(&mut j, &input_values, &mut pw)?;
                }
                _ => {
                    let feature_name = format!("parameter {:?}", value);
                    return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                }
            }
        }

        let proof = match self.data.prove(pw) {
            Ok(proof) => proof,
            Err(error) => {
                let error_message = format!("Unexpected proof failure: {:?}", error);
                return Err(Plonky2GenError::ICE { message: error_message });
            }
        };
        let proof_serialized = match serde_json::to_vec(&proof) {
            Ok(bytes) => bytes,
            Err(error) => {
                let error_message = format!("Unexpected serialization error: {:?}", error);
                return Err(Plonky2GenError::ICE { message: error_message });
            }
        };
        Ok(proof_serialized)
    }

    fn verify_public_inputs_in_proof(
        public_inputs: WitnessMap<FieldElement>,
        proof: &P2ProofWithPublicInputs,
    ) -> bool {
        let mut next_index: usize = 0;
        for (witness, element) in public_inputs {
            if witness != acvm::acir::native_types::Witness(next_index.try_into().unwrap()) {
                return false;
            }
            if noir_to_plonky2_field(element) != proof.public_inputs[next_index] {
                return false;
            }
            next_index += 1;
        }
        if proof.public_inputs.len() != next_index {
            return false;
        }
        true
    }

    pub fn verify(
        &self,
        proof: &[u8],
        public_inputs: WitnessMap<FieldElement>,
    ) -> Result<bool, Plonky2VerifyError> {
        let deserialized_proof: P2ProofWithPublicInputs = match serde_json::from_slice(proof) {
            Ok(deserialized_proof) => deserialized_proof,
            Err(error) => {
                let error_message = format!("Unexpected deserialization error: {:?}", error);
                return Err(Plonky2VerifyError::VerificationFailed { message: error_message });
            }
        };
        if !Self::verify_public_inputs_in_proof(public_inputs, &deserialized_proof) {
            return Err(Plonky2VerifyError::VerificationFailed {
                message: "Public inputs don't match proof".to_string(),
            });
        }
        match self.data.verify(deserialized_proof) {
            Ok(_) => Ok(true),
            Err(error) => {
                let error_message = format!("Unexpected proof verification failure: {:?}", error);
                Err(Plonky2VerifyError::VerificationFailed { message: error_message })
            }
        }
    }

    fn set_parameter(&self, j: &mut usize, field: FieldElement, pw: &mut PartialWitness<P2Field>) {
        let target = self.parameters[*j];
        *j += 1;
        let field = noir_to_plonky2_field(field);
        pw.set_target(target, field)
    }

    fn set_array_parameter(
        &self,
        j: &mut usize,
        input_values: &Vec<noirc_abi::input_parser::InputValue>,
        pw: &mut PartialWitness<P2Field>,
    ) -> Result<(), Plonky2GenError> {
        for input_value in input_values.iter() {
            match input_value {
                InputValue::Field(field) => {
                    self.set_parameter(j, *field, pw);
                }

                InputValue::Vec(input_values) => {
                    let _ = self.set_array_parameter(j, input_values, pw)?;
                }

                InputValue::Struct(fields) => {
                    let mut input_values = Vec::new();
                    for value in fields.values() {
                        input_values.push(value.clone());
                    }
                    let _ = self.set_array_parameter(j, &input_values, pw)?;
                }

                _ => {
                    let feature_name = format!("array parameter with non-field elements");
                    return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                }
            }
        }
        Ok(())
    }
}
