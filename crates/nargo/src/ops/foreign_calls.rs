use acvm::{
    acir::brillig::{ForeignCallResult, Value},
    pwg::ForeignCallWaitInfo,
};
use iter_extended::vecmap;
use noirc_abi::{decode_string_value, decode_value, input_parser::json::JsonTypes, AbiType};

use crate::errors::ForeignCallError;

/// This enumeration represents the Brillig foreign calls that are natively supported by nargo.
/// After resolution of a foreign call, nargo will restart execution of the ACVM
pub(crate) enum ForeignCall {
    Println,
    Sequence,
    ReverseSequence,
}

impl std::fmt::Display for ForeignCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ForeignCall {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            ForeignCall::Println => "println",
            ForeignCall::Sequence => "get_number_sequence",
            ForeignCall::ReverseSequence => "get_reverse_number_sequence",
        }
    }

    pub(crate) fn lookup(op_name: &str) -> Option<ForeignCall> {
        match op_name {
            "println" => Some(ForeignCall::Println),
            "get_number_sequence" => Some(ForeignCall::Sequence),
            "get_reverse_number_sequence" => Some(ForeignCall::ReverseSequence),
            _ => None,
        }
    }

    pub(crate) fn execute(
        foreign_call: &ForeignCallWaitInfo,
    ) -> Result<ForeignCallResult, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match Self::lookup(foreign_call_name) {
            Some(ForeignCall::Println) => {
                Self::execute_println(&foreign_call.inputs)?;
                Ok(ForeignCallResult { values: vec![] })
            }
            Some(ForeignCall::Sequence) => {
                let sequence_length: u128 = foreign_call.inputs[0][0].to_field().to_u128();

                Ok(vecmap(0..sequence_length, Value::from).into())
            }
            Some(ForeignCall::ReverseSequence) => {
                let sequence_length: u128 = foreign_call.inputs[0][0].to_field().to_u128();

                Ok(vecmap((0..sequence_length).rev(), Value::from).into())
            }
            None => panic!("unexpected foreign call {:?}", foreign_call_name),
        }
    }

    fn execute_println(foreign_call_inputs: &[Vec<Value>]) -> Result<(), ForeignCallError> {
        let (abi_type, input_values) = fetch_abi_type(foreign_call_inputs)?;

        // We must use a flat map here as each value in a struct will be in a separate input value
        let mut input_values_as_fields =
            input_values.iter().flat_map(|values| values.iter().map(|value| value.to_field()));
        let decoded_value = decode_value(&mut input_values_as_fields, &abi_type)?;

        let json_value = JsonTypes::try_from_input_value(&decoded_value, &abi_type)?;

        println!("{json_value}");
        Ok(())
    }
}

/// Fetch the abi type from the foreign call input
/// The remaining input values should hold the values to be printed
fn fetch_abi_type(
    foreign_call_inputs: &[Vec<Value>],
) -> Result<(AbiType, &[Vec<Value>]), ForeignCallError> {
    let (abi_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;
    let abi_type_as_fields = vecmap(abi_type_as_values, |value| value.to_field());
    let abi_type_as_string = decode_string_value(&abi_type_as_fields);
    let abi_type: AbiType = serde_json::from_str(&abi_type_as_string)
        .map_err(|err| ForeignCallError::InputParserError(err.into()))?;

    Ok((abi_type, input_values))
}
