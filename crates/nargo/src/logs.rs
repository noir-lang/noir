use crate::CliError;
use acvm::{
    acir::circuit::directives::{SolvedLog, SolvedLogOutputInfo},
    FieldElement,
};
use iter_extended::vecmap;

pub(crate) fn handle_logs(logs: Vec<SolvedLog>) -> Result<(), CliError> {
    for log in logs {
        println!("is_trace: ");
        dbg!(log.is_trace);
        let output_string = match log.output_info {
            SolvedLogOutputInfo::FinalizedOutput(output_string) => output_string.clone(),
            SolvedLogOutputInfo::WitnessValues(field_elements) => {
                if field_elements.len() == 1 {
                    let element = &field_elements[0];
                    let output_string = format_field_string(*element);
                    output_string
                } else {
                    // If multiple field elements are fetched for a solved log,
                    // it assumed that an array is meant to be printed to standard output
                    //
                    // Collect all field element values corresponding to the given witness indices (whose values were solved during PWG)
                    // and convert them to hex strings.
                    let elements_as_hex = vecmap(field_elements, |e| format_field_string(e));
                    let comma_separated_elements = elements_as_hex.join(", ");

                    let output_witnesses_string = "[".to_owned() + &comma_separated_elements + "]";
                    output_witnesses_string
                }
            }
        };
        if !log.is_trace {
            println!("{output_string}")
        } else {
            // TODO: Decide how we want to handle traces
            // They are used during `nargo execute` and `nargo test` which do not have explicit program or proof names associated with them
            // For now nargo has them keep the same functionality as println, but they are useful for any JS developer who needs to keep things in memory
            println!("{output_string}");
        }
    }
    Ok(())
}

/// This trims any leading zeroes.
/// A singular '0' will be prepended as well if the trimmed string has an odd length.
/// A hex string's length needs to be even to decode into bytes, as two digits correspond to
/// one byte.
fn format_field_string(field: FieldElement) -> String {
    let mut trimmed_field = field.to_hex().trim_start_matches('0').to_owned();
    if trimmed_field.len() % 2 != 0 {
        trimmed_field = "0".to_owned() + &trimmed_field
    };
    "0x".to_owned() + &trimmed_field
}
