use acvm::{
    acir::brillig::{ForeignCallOutput, ForeignCallResult, Value},
    pwg::ForeignCallWaitInfo,
};
use iter_extended::vecmap;
use noirc_printable_type::PrintableValueDisplay;

use crate::NargoError;

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
    ) -> Result<ForeignCallResult, NargoError> {
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

    fn execute_println(foreign_call_inputs: &[Vec<Value>]) -> Result<(), NargoError> {
        let display_values: PrintableValueDisplay = foreign_call_inputs.try_into()?;
        println!("{display_values}");
        Ok(())
    }
}
