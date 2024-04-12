use super::config::P2CircuitData;
use super::noir_to_plonky2_field;
use noirc_abi::InputMap;
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
    pub fn prove(&self, inputs: &InputMap) -> Option<Vec<u8>> {
        let mut pw = PartialWitness::new();
        for (target, param_name) in self.parameters.iter().zip(&self.parameter_names) {
            let value = inputs[param_name].clone();

            match value {
                noirc_abi::input_parser::InputValue::Field(field) => {
                    let field = noir_to_plonky2_field(field);
                    pw.set_target(*target, field)
                }
                _ => todo!(),
            }
        }

        let proof = self.data.prove(pw).ok()?;
        let proof_seiralized = serde_json::to_vec(&proof).ok()?;
        Some(proof_seiralized)
    }
}
