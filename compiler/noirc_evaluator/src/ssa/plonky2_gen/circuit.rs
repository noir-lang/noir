use super::config::{P2CircuitData, P2Field};
use super::noir_to_plonky2_field;
use crate::errors::Plonky2GenError;
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
}

impl Clone for Plonky2Circuit {
    fn clone(&self) -> Self {
        todo!()
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
                _ => {
                    let feature_name = format!("array parameter with non-field elements");
                    return Err(Plonky2GenError::UnsupportedFeature { name: feature_name });
                }
            }
        }
        Ok(())
    }
}
