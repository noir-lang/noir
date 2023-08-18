use acvm::{
    acir::brillig::{ForeignCallOutput, ForeignCallResult, Value},
    pwg::ForeignCallWaitInfo,
};
use iter_extended::vecmap;
use noirc_printable_type::{
    decode_string_value, decode_value, PrintableType, PrintableValueDisplay,
};
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
        show_output: bool,
    ) -> Result<ForeignCallResult, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match Self::lookup(foreign_call_name) {
            Some(ForeignCall::Println) => {
                if show_output {
                    Self::execute_println(&foreign_call.inputs)?;
                }
                Ok(ForeignCallResult { values: vec![] })
            }
            Some(ForeignCall::Sequence) => {
                let sequence_length: u128 = foreign_call.inputs[0][0].to_field().to_u128();
                let sequence = vecmap(0..sequence_length, Value::from);

                Ok(ForeignCallResult {
                    values: vec![
                        ForeignCallOutput::Single(sequence_length.into()),
                        ForeignCallOutput::Array(sequence),
                    ],
                })
            }
            Some(ForeignCall::ReverseSequence) => {
                let sequence_length: u128 = foreign_call.inputs[0][0].to_field().to_u128();
                let sequence = vecmap((0..sequence_length).rev(), Value::from);

                Ok(ForeignCallResult {
                    values: vec![
                        ForeignCallOutput::Single(sequence_length.into()),
                        ForeignCallOutput::Array(sequence),
                    ],
                })
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
    // Fetch the PrintableType from the foreign call input
    // The remaining input values should hold what is to be printed
    let (printable_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;
    let printable_type = fetch_printable_type(printable_type_as_values)?;

    // We must use a flat map here as each value in a struct will be in a separate input value
    let mut input_values_as_fields =
        input_values.iter().flat_map(|values| vecmap(values, |value| value.to_field()));

    let value = decode_value(&mut input_values_as_fields, &printable_type);
    let input_value_display = PrintableValueDisplay::new(&value, &printable_type);

    Ok(input_value_display.to_string())
}

fn convert_fmt_string_inputs(
    foreign_call_inputs: &[Vec<Value>],
) -> Result<String, ForeignCallError> {
    let (message_as_values, input_and_printable_values) =
        foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let message_as_fields = vecmap(message_as_values, |value| value.to_field());
    let message_as_string = decode_string_value(&message_as_fields);

    let (num_values, input_and_printable_values) = input_and_printable_values
        .split_first()
        .ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let mut output_strings = Vec::new();
    let num_values = num_values[0].to_field().to_u128() as usize;

    let mut printable_types = Vec::new();
    for printable_value in
        input_and_printable_values.iter().skip(input_and_printable_values.len() - num_values)
    {
        let printable_type = fetch_printable_type(printable_value)?;
        printable_types.push(printable_type);
    }

    for i in 0..num_values {
        let printable_type = &printable_types[i];
        let type_size = printable_type.field_count() as usize;

        let mut input_values_as_fields = input_and_printable_values[i..(i + type_size)]
            .iter()
            .flat_map(|values| vecmap(values, |value| value.to_field()));

        let value = decode_value(&mut input_values_as_fields, printable_type);
        let input_value_display = PrintableValueDisplay::new(&value, printable_type);

        output_strings.push(input_value_display.to_string());
    }

    let mut output_strings_iter = output_strings.into_iter();
    let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}")
        .expect("ICE: an invalid regex pattern was used for checking format strings");

    let formatted_str = re.replace_all(&message_as_string, |_: &Captures| {
        output_strings_iter
            .next()
            .expect("ICE: there are more regex matches than fields supplied to the format string")
    });

    Ok(formatted_str.into_owned())
}

fn fetch_printable_type(
    printable_type_as_values: &[Value],
) -> Result<PrintableType, ForeignCallError> {
    let printable_type_as_fields = vecmap(printable_type_as_values, |value| value.to_field());
    let printable_type_as_string = decode_string_value(&printable_type_as_fields);
    let printable_type: PrintableType = serde_json::from_str(&printable_type_as_string)
        .map_err(|err| ForeignCallError::InputParserError(err.into()))?;

    Ok(printable_type)
}
