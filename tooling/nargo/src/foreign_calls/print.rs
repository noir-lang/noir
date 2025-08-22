use acvm::{AcirField, acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo};
use noirc_printable_type::PrintableValueDisplay;

use super::{ForeignCall, ForeignCallError, ForeignCallExecutor};

/// Handle `println` calls.
#[derive(Debug, Default)]
pub struct PrintForeignCallExecutor<W> {
    output: W,
}

impl<W> PrintForeignCallExecutor<W> {
    pub fn new(output: W) -> Self {
        Self { output }
    }
}

impl<F: AcirField, W: std::io::Write> ForeignCallExecutor<F> for PrintForeignCallExecutor<W> {
    /// Print has certain information encoded in the call arguments.
    /// Below we outline the expected inputs.
    ///
    /// For regular printing:
    /// [print_newline][acvm::acir::brillig::ForeignCallParam::Single]: 0 for print, 1 for println
    /// [value_to_print][acvm::acir::brillig::ForeignCallParam::Array]: The field values representing the actual value to print
    /// [type_metadata][acvm::acir::brillig::ForeignCallParam::Array]: Field values representing the JSON encoded type, which tells us how to print the above value
    /// [is_fmt_str][acvm::acir::brillig::ForeignCallParam::Single]: 0 for regular string, 1 for indicating we have a format string
    ///
    /// For printing a format string:
    /// [print_newline][acvm::acir::brillig::ForeignCallParam::Single]: 0 for print, 1 for println
    /// [message][acvm::acir::brillig::ForeignCallParam::Array]: The fmtstr as a regular string
    /// [num_values][acvm::acir::brillig::ForeignCallParam::Single]: Number of values in the fmtstr
    /// [[value_to_print][acvm::acir::brillig::ForeignCallParam::Array]; N]: Array of the field values for each value in the fmtstr
    /// [[type_metadata][acvm::acir::brillig::ForeignCallParam::Array]; N]: Array of field values representing the JSON encoded types
    /// [is_fmt_str][acvm::acir::brillig::ForeignCallParam::Single]: 0 for regular string, 1 for indicating we have a format string
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

                let display_values =
                    PrintableValueDisplay::<F>::try_from_params(foreign_call_inputs)?;

                if skip_newline {
                    write!(self.output, "{display_values}").expect("write should succeed");
                } else {
                    writeln!(self.output, "{display_values}").expect("write should succeed");
                }

                Ok(ForeignCallResult::default())
            }
            _ => Err(ForeignCallError::NoHandler(foreign_call_name.to_string())),
        }
    }
}
