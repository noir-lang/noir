use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{create_named_dir, write_to_file};
use crate::CliError;
use acvm::acir::circuit::directives::{SolvedLog, SolvedLogOutputInfo};
use iter_extended::vecmap;
use noirc_abi::format_field_string;

pub(crate) fn handle_logs<P: AsRef<Path>>(
    logs: Vec<SolvedLog>,
    debug_file_name: Option<String>,
    path: P,
) -> Result<(), CliError> {
    let mut traces: HashMap<String, String> = HashMap::new();
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
            // We can have multiples traces with the same label.
            // Below we group traces into a singular list containing all traces with a specific label
            if let Some(val) = traces.get_mut(&trace_label) {
                // If there are multiples traces with the same label we want to maintain the order of the first insertion.
                // Thus, we alter the value in the traces map rather than inserting a new value
                *val = val.clone() + ", " + &output_string;
            } else {
                traces.insert(trace_label, output_string);
            };
        } else {
            println!("{output_string}")
        }
    }

    match debug_file_name {
        Some(file_name) if !traces.is_empty() => {
            let mut debug_dir = PathBuf::from(path.as_ref());
            debug_dir.push("debug");
            let mut trace_path = create_named_dir(debug_dir.as_ref(), "debug");
            trace_path.push(file_name);
            trace_path.set_extension("trace");

            write_to_file(&serde_json::to_vec(&traces).unwrap(), &trace_path);
        }
        _ => (),
    }

    Ok(())
}
