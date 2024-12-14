use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use noirc_printable_type::{ForeignCallError, PrintableValueDisplay};

use super::{ForeignCall, ForeignCallExecutor};

#[derive(Debug, Default)]
pub enum PrintOutput<'a> {
    #[default]
    None,
    Stdout,
    String(&'a mut String),
}

#[derive(Debug, Default)]
pub(crate) struct PrintForeignCallExecutor<'a> {
    pub(crate) output: PrintOutput<'a>,
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

                let display_values: PrintableValueDisplay<F> = foreign_call_inputs.try_into()?;
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
