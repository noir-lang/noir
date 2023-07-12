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
    PrintlnFormat,
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
            ForeignCall::PrintlnFormat => "println_format",
            ForeignCall::Sequence => "get_number_sequence",
            ForeignCall::ReverseSequence => "get_reverse_number_sequence",
        }
    }

    pub(crate) fn lookup(op_name: &str) -> Option<ForeignCall> {
        match op_name {
            "println" => Some(ForeignCall::Println),
            "println_format" => Some(ForeignCall::PrintlnFormat),
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
            Some(ForeignCall::PrintlnFormat) => {
                Self::execute_println_format(&foreign_call.inputs)?;
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
        // Fetch the abi type from the foreign call input
        let (abi_type, input_values) = fetch_abi_type(foreign_call_inputs)?;

        let input_values_as_fields = input_values
            .into_iter()
            .flat_map(|values| vecmap(values, |value| value.to_field()))
            .collect::<Vec<_>>();
        let decoded_value = decode_value(&mut input_values_as_fields.into_iter(), &abi_type)
            .map_err(|err| ForeignCallError::AbiError(err))?;

        let json_value = JsonTypes::try_from_input_value(&decoded_value, &abi_type)
            .map_err(|err| ForeignCallError::InputParserError(err))?;
        let output_string = serde_json::to_string_pretty(&json_value)
            .map_err(|err| ForeignCallError::InputParserError(err.into()))?;

        println!("{output_string}");
        Ok(())
    }

    fn execute_println_format(foreign_call_inputs: &[Vec<Value>]) -> Result<(), ForeignCallError> {
        // Fetch the message from the first input
        let (message_as_values, input_and_abi_values) =
            foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;
        // Fetch the abi type from the foreign call input
        let (abi_type, input_values) = fetch_abi_type(input_and_abi_values)?;

        let input_values_as_fields = vecmap(input_values[0].iter(), |value| value.to_field());

        let mut output_strings = Vec::new();

        // This currently only works for arrays of single values
        for input_value in input_values_as_fields {
            let value_to_decode = vec![input_value];
            let decoded_value = decode_value(&mut value_to_decode.into_iter(), &abi_type)
                .map_err(|err| ForeignCallError::AbiError(err))?;
            let json_value = JsonTypes::try_from_input_value(&decoded_value, &abi_type)
                .map_err(|err| ForeignCallError::InputParserError(err))?;
            let output_string = serde_json::to_string_pretty(&json_value)
                .map_err(|err| ForeignCallError::InputParserError(err.into()))?;
            output_strings.push(output_string);
        }

        let message_as_fields = vecmap(message_as_values, |value| value.to_field());
        let message_as_string = decode_string_value(&message_as_fields);

        let re = Regex::new(r"\{(\d+)\}").unwrap();

        let formatted_str = re.replace_all(&message_as_string, |caps: &Captures| {
            let (_, [target_idx]) = caps.extract();
            &output_strings[target_idx.parse::<usize>().unwrap()]
        });
        println!("{formatted_str}");
        Ok(())
    }
}

fn fetch_abi_type(
    foreign_call_inputs: &[Vec<Value>],
) -> Result<(AbiType, &[Vec<Value>]), ForeignCallError> {
    // Fetch the abi from the last input. We will now be left with
    let (abi_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;
    let abi_type_as_fields = vecmap(abi_type_as_values, |value| value.to_field());
    let abi_type_as_string = decode_string_value(&abi_type_as_fields);
    let abi_type: AbiType = serde_json::from_str(&abi_type_as_string)
        .map_err(|err| ForeignCallError::InputParserError(err.into()))?;

    Ok((abi_type, input_values))
}
