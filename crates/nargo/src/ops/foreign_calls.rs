use acvm::{
    acir::brillig::{ForeignCallResult, Value},
    pwg::ForeignCallWaitInfo,
};
use iter_extended::vecmap;
use noirc_abi::{decode_string_value, decode_value, input_parser::json::JsonTypes, AbiType};
use regex::{Captures, Regex};

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
                Ok(foreign_call.inputs[0][0].into())
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
        let (is_fmt_str, foreign_call_inputs) =
            foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;

        let output_string = if is_fmt_str[0].to_field().is_one() {
            convert_fmt_string_inputs(foreign_call_inputs)?
        } else {
            convert_string_inputs(foreign_call_inputs)?
        };
        println!("{output_string}");
        Ok(())
    }
}

fn convert_string_inputs(foreign_call_inputs: &[Vec<Value>]) -> Result<String, ForeignCallError> {
    // Fetch the abi type from the foreign call input
    // The remaining input values should hold what is to be printed
    let (abi_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;
    let abi_type = fetch_abi_type(abi_type_as_values)?;

    // We must use a flat map here as each value in a struct will be in a separate input value
    let mut input_values_as_fields =
        input_values.iter().flat_map(|values| vecmap(values, |value| value.to_field()));

    let decoded_value = decode_value(&mut input_values_as_fields, &abi_type)?;
    let json_value = JsonTypes::try_from_input_value(&decoded_value, &abi_type)?;

    serde_json::to_string_pretty(&json_value)
        .map_err(|err| ForeignCallError::InputParserError(err.into()))
}

fn convert_fmt_string_inputs(
    foreign_call_inputs: &[Vec<Value>],
) -> Result<String, ForeignCallError> {
    let (message_as_values, input_and_abi_values) =
        foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let message_as_fields = vecmap(message_as_values, |value| value.to_field());
    let message_as_string = decode_string_value(&message_as_fields);

    let (num_values, input_and_abi_values) =
        input_and_abi_values.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let mut output_strings = Vec::new();
    let num_values = num_values[0].to_field().to_u128() as usize;

    let mut abi_types = Vec::new();
    for i in (input_and_abi_values.len() - num_values)..input_and_abi_values.len() {
        let abi_type = fetch_abi_type(&input_and_abi_values[i])?;
        abi_types.push(abi_type);
    }

    for i in 0..num_values {
        let abi_type = &abi_types[i];
        let type_size = abi_type.field_count() as usize;

        let input_values_as_fields = input_and_abi_values[i..(i + type_size)]
            .iter()
            .flat_map(|values| vecmap(values, |value| value.to_field()));

        let decoded_value = decode_value(&mut input_values_as_fields.into_iter(), abi_type)?;

        let json_value = JsonTypes::try_from_input_value(&decoded_value, abi_type)?;
        let output_string = serde_json::to_string(&json_value)
            .map_err(|err| ForeignCallError::InputParserError(err.into()))?;
        output_strings.push(output_string);
    }

    let mut output_strings_iter = output_strings.into_iter();
    let re = Regex::new(r"\{([a-zA-Z0-9]+)\}").unwrap();

    let formatted_str =
        re.replace_all(&message_as_string, |_: &Captures| output_strings_iter.next().unwrap());

    Ok(formatted_str.into_owned())
}

fn fetch_abi_type(abi_type_as_values: &[Value]) -> Result<AbiType, ForeignCallError> {
    let abi_type_as_fields = vecmap(abi_type_as_values, |value| value.to_field());
    let abi_type_as_string = decode_string_value(&abi_type_as_fields);
    let abi_type: AbiType = serde_json::from_str(&abi_type_as_string)
        .map_err(|err| ForeignCallError::InputParserError(err.into()))?;

    Ok(abi_type)
}
