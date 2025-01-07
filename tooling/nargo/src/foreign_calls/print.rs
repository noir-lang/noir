use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult},
    pwg::ForeignCallWaitInfo,
    AcirField,
};
use noirc_abi::{decode_printable_value, decode_string_value};
use noirc_printable_type::{PrintableType, PrintableValueDisplay};

use super::{ForeignCall, ForeignCallError, ForeignCallExecutor};

#[derive(Debug, Default)]
pub enum PrintOutput<'a> {
    #[default]
    None,
    Stdout,
    String(&'a mut String),
}

#[derive(Debug, Default)]
pub struct PrintForeignCallExecutor<'a> {
    output: PrintOutput<'a>,
}

impl<'a> PrintForeignCallExecutor<'a> {
    pub fn new(output: PrintOutput<'a>) -> Self {
        Self { output }
    }
}

impl<F: AcirField> ForeignCallExecutor<F> for PrintForeignCallExecutor<'_> {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match ForeignCall::lookup(foreign_call_name) {
            Some(ForeignCall::Print) => {
                let skip_newline = foreign_call.inputs[0].unwrap_field().is_zero();

                let foreign_call_inputs = foreign_call
                    .inputs
                    .split_first()
                    .ok_or(ForeignCallError::MissingForeignCallInputs)?
                    .1;

                let display_values: PrintableValueDisplay<F> =
                    try_from_params(foreign_call_inputs)?;
                let display_string =
                    format!("{display_values}{}", if skip_newline { "" } else { "\n" });

                match &mut self.output {
                    PrintOutput::None => (),
                    PrintOutput::Stdout => print!("{display_string}"),
                    PrintOutput::String(string) => {
                        string.push_str(&display_string);
                    }
                }

                Ok(ForeignCallResult::default())
            }
            _ => Err(ForeignCallError::NoHandler(foreign_call_name.to_string())),
        }
    }
}

fn try_from_params<F: AcirField>(
    foreign_call_inputs: &[ForeignCallParam<F>],
) -> Result<PrintableValueDisplay<F>, ForeignCallError> {
    let (is_fmt_str, foreign_call_inputs) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;

    if is_fmt_str.unwrap_field().is_one() {
        convert_fmt_string_inputs(foreign_call_inputs)
    } else {
        convert_string_inputs(foreign_call_inputs)
    }
}

fn convert_string_inputs<F: AcirField>(
    foreign_call_inputs: &[ForeignCallParam<F>],
) -> Result<PrintableValueDisplay<F>, ForeignCallError> {
    // Fetch the PrintableType from the foreign call input
    // The remaining input values should hold what is to be printed
    let (printable_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;
    let printable_type = fetch_printable_type(printable_type_as_values)?;

    // We must use a flat map here as each value in a struct will be in a separate input value
    let mut input_values_as_fields = input_values.iter().flat_map(|param| param.fields());

    let value = decode_printable_value(&mut input_values_as_fields, &printable_type);

    Ok(PrintableValueDisplay::Plain(value, printable_type))
}

fn convert_fmt_string_inputs<F: AcirField>(
    foreign_call_inputs: &[ForeignCallParam<F>],
) -> Result<PrintableValueDisplay<F>, ForeignCallError> {
    let (message, input_and_printable_types) =
        foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let message_as_fields = message.fields();
    let message_as_string = decode_string_value(&message_as_fields);

    let (num_values, input_and_printable_types) = input_and_printable_types
        .split_first()
        .ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let mut output = Vec::new();
    let num_values = num_values.unwrap_field().to_u128() as usize;

    let types_start_at = input_and_printable_types.len() - num_values;
    let mut input_iter =
        input_and_printable_types[0..types_start_at].iter().flat_map(|param| param.fields());
    for printable_type in input_and_printable_types.iter().skip(types_start_at) {
        let printable_type = fetch_printable_type(printable_type)?;
        let value = decode_printable_value(&mut input_iter, &printable_type);

        output.push((value, printable_type));
    }

    Ok(PrintableValueDisplay::FmtString(message_as_string, output))
}

fn fetch_printable_type<F: AcirField>(
    printable_type: &ForeignCallParam<F>,
) -> Result<PrintableType, ForeignCallError> {
    let printable_type_as_fields = printable_type.fields();
    let printable_type_as_string = decode_string_value(&printable_type_as_fields);
    let printable_type: PrintableType = serde_json::from_str(&printable_type_as_string)?;

    Ok(printable_type)
}
