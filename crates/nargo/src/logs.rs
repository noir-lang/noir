use crate::CliError;
use acvm::acir::circuit::directives::{SolvedLog, SolvedLogOutputInfo};
use iter_extended::vecmap;
use noirc_abi::format_field_string;

pub(crate) fn handle_logs(logs: Vec<SolvedLog>) -> Result<(), CliError> {
    for log in logs {
        let output_string = match log.output_info {
            SolvedLogOutputInfo::FinalizedOutput(output_string) => output_string.clone(),
            SolvedLogOutputInfo::WitnessValues(field_elements) => {
                if field_elements.len() == 1 {
                    let element = &field_elements[0];
                    format_field_string(*element)
                } else {
                    // If multiple field elements are fetched for a solved log,
                    // it assumed that an array is meant to be printed to standard output
                    //
                    // Collect all field element values corresponding to the given witness indices (whose values were solved during PWG)
                    // and convert them to hex strings.
                    let elements_as_hex = vecmap(field_elements, format_field_string);
                    let comma_separated_elements = elements_as_hex.join(", ");

                    "[".to_owned() + &comma_separated_elements + "]"
                }
            }
        };
        if let Some(trace_label) = log.trace_label {
            write_to_file
        } else {
            println!("{output_string}")
        }

        // if !log.is_trace {
        //     println!("{output_string}")
        // } else {
        //     // TODO: Decide how we want to handle traces
        //     // They are used during `nargo execute` and `nargo test` which do not have explicit program or proof names associated with them
        //     // For now nargo has them keep the same functionality as println, but they are useful for any JS developer who needs to keep things in memory
        //     println!("{output_string}");
        // }
    }
    Ok(())
}
