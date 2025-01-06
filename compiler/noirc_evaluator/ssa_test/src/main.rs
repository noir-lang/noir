use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::path::Path;
use flate2::read::GzDecoder;
use::noirc_evaluator::acir_instruction_builder::{
    InstructionArtifacts, VariableType, Variable
};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "../../../../../barretenberg/cpp/src/barretenberg/acir_formal_proofs/artifacts/")]
    dir: String,
}

/// Decompresses gzipped data into a byte vector
fn ungzip(compressed_data: Vec<u8>) -> Vec<u8> {
    let mut decompressed_data: Vec<u8> = Vec::new();
    let mut decoder = GzDecoder::new(&compressed_data[..]);
    decoder.read_to_end(&mut decompressed_data).unwrap();
    return decompressed_data;
}

/// Saves byte data to a file at the specified path
fn save_to_file(data: &[u8], filename: &str) -> Result<(), std::io::Error> {
    let path = Path::new(filename);
    let mut file = File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

/// Saves instruction artifacts to files in the artifacts directory
/// Prints the formatted SSA for each artifact and saves the decompressed ACIR
fn save_artifacts(all_artifacts: Vec<InstructionArtifacts>, dir: &str) {
    for artifacts in all_artifacts.iter() {
        println!("{}", artifacts.formatted_ssa);
        let filename = format!("{}{}{}", dir, artifacts.instruction_name, ".acir");
        let acir = &artifacts.serialized_acir;
        match save_to_file(&ungzip(acir.clone()), &filename) {
            Ok(_) => (),
            Err(error) => println!("Error saving data: {}", error),
        }
    }
}

/// Main function that generates test artifacts for SSA instructions
/// Creates test cases for various operations with different variable types and bit sizes
fn main() {
    let args = Args::parse();

    let mut all_artifacts: Vec<InstructionArtifacts> = Vec::new();

    // Define test variables with different types and sizes
    let field_var = Variable{ variable_type: VariableType::Field, variable_size: 0};
    // max bit size for signed and unsigned
    let u127_var = Variable{ variable_type: VariableType::Unsigned, variable_size: 127};
    let i127_var = Variable{ variable_type: VariableType::Signed, variable_size: 127};
    // max bit size allowed by mod and div
    let u126_var = Variable{ variable_type: VariableType::Unsigned, variable_size: 126};
    let i126_var = Variable{ variable_type: VariableType::Signed, variable_size: 126};
    // 64 bit unsigned
    let u64_var = Variable{ variable_type: VariableType::Unsigned, variable_size: 64};
    // 32 bit unsigned
    let u32_var = Variable{ variable_type: VariableType::Unsigned, variable_size: 32};
    // 8 bit unsigned
    let u8_var = Variable{ variable_type: VariableType::Unsigned, variable_size: 8};

    // Test bitvector operations with max bit size (127 bits)
    all_artifacts.push(InstructionArtifacts::new_add(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_sub(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_mul(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_mod(&u126_var, &u126_var));
    all_artifacts.push(InstructionArtifacts::new_xor(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_and(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_div(&u126_var, &u126_var));
    all_artifacts.push(InstructionArtifacts::new_eq(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_lt(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_xor(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_or(&u127_var, &u127_var));
    all_artifacts.push(InstructionArtifacts::new_not(&u127_var));
    all_artifacts.push(InstructionArtifacts::new_constrain(&u127_var));
    all_artifacts.push(InstructionArtifacts::new_truncate(&u127_var));
    all_artifacts.push(InstructionArtifacts::new_range_check(&u127_var));

    // Test shift operations with smaller bit sizes
    // shl truncates variable, so test different sizes
    // Too heavy to test 127 bits, but it just multiplies or divides by 2^rhs
    // Should work the same if div and mul are verified
    all_artifacts.push(InstructionArtifacts::new_shl(&u64_var, &u8_var));
    all_artifacts.push(InstructionArtifacts::new_shr(&u64_var, &u8_var));
    all_artifacts.push(InstructionArtifacts::new_shl(&u32_var, &u8_var));

    // Test binary operations with 32 bits
    all_artifacts.push(InstructionArtifacts::new_xor(&u32_var, &u32_var));
    all_artifacts.push(InstructionArtifacts::new_and(&u32_var, &u32_var));
    all_artifacts.push(InstructionArtifacts::new_or(&u32_var, &u32_var));

    // Test field operations
    all_artifacts.push(InstructionArtifacts::new_add(&field_var, &field_var));
    all_artifacts.push(InstructionArtifacts::new_mul(&field_var, &field_var));
    all_artifacts.push(InstructionArtifacts::new_div(&field_var, &field_var));
    all_artifacts.push(InstructionArtifacts::new_eq(&field_var, &field_var));

    // Test signed division (only operation that differs for signed integers)
    all_artifacts.push(InstructionArtifacts::new_div(&i126_var, &i126_var));
    all_artifacts.push(InstructionArtifacts::new_lt(&i127_var, &i127_var));

    save_artifacts(all_artifacts, &args.dir);
}
