#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

mod acir_instruction_builder;
use crate::acir_instruction_builder::{InstructionArtifacts, Variable, VariableType};
use clap::Parser;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

/// Command line arguments for the SSA test generator
#[derive(Parser)]
#[command(
    author,
    version,
    about = "Generates test artifacts for formally verifying SSA instructions and their conversion to ACIR",
    long_about = "This tool generates test cases for various operations including:
- Bitvector operations (up to 128 bits): add, sub, mul, mod, xor, and, div, eq, lt, not
- Shift operations (32 and 64 bits): shl, shr  
- Binary operations (32-bit): xor, and, or
- Field operations: add, mul, div

Each test case generates formatted SSA representation and serialized ACIR output."
)]
struct Args {
    /// Output directory path for the generated test artifacts
    /// Defaults to the barretenberg acir formal proofs artifacts directory
    #[arg(short, long, default_value = "/tmp/")]
    dir: String,
}

/// Decompresses gzipped data into a byte vector
fn ungzip(compressed_data: &[u8]) -> Vec<u8> {
    let mut decompressed_data: Vec<u8> = Vec::new();
    let mut decoder = GzDecoder::new(compressed_data);
    decoder.read_to_end(&mut decompressed_data).unwrap();
    decompressed_data
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
        println!("{}\n{}", artifacts.instruction_name, artifacts.formatted_ssa);
        let filename = format!("{}{}{}", dir, artifacts.instruction_name, ".acir");
        let acir = &artifacts.serialized_acir;
        match save_to_file(&ungzip(acir), &filename) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("Error saving data: {error}");
                std::process::exit(1);
            }
        }
    }
}

/// Main function that generates test artifacts for SSA instructions
/// Creates test cases for various operations with different variable types and bit sizes
fn main() {
    let args = Args::parse();

    let mut all_artifacts: Vec<InstructionArtifacts> = Vec::new();

    // Define test variables with different types and sizes
    let field_var = Variable { variable_type: VariableType::Field, variable_size: 0 };
    // max bit size for signed and unsigned
    let u128_var = Variable { variable_type: VariableType::Unsigned, variable_size: 128 };
    // 64 bit unsigned
    let u64_var = Variable { variable_type: VariableType::Unsigned, variable_size: 64 };
    // 32 bit unsigned
    let u32_var = Variable { variable_type: VariableType::Unsigned, variable_size: 32 };

    let u8_var = Variable { variable_type: VariableType::Unsigned, variable_size: 8 };
    let i8_var = Variable { variable_type: VariableType::Signed, variable_size: 8 };

    let i64_var = Variable { variable_type: VariableType::Signed, variable_size: 64 };

    // Tests for bitvector operations with max bit size (128 bits)
    all_artifacts.push(InstructionArtifacts::new_add(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_sub(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_mul(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_mod(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_xor(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_and(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_div(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_eq(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_lt(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_xor(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_or(&u128_var, &u128_var));
    all_artifacts.push(InstructionArtifacts::new_not(&u128_var));
    all_artifacts.push(InstructionArtifacts::new_constrain(&u128_var));
    all_artifacts.push(InstructionArtifacts::new_range_check(&u128_var, 64));

    all_artifacts.push(InstructionArtifacts::new_add(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_sub(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_mul(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_mod(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_xor(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_and(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_div(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_eq(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_lt(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_xor(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_or(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_not(&i64_var));
    all_artifacts.push(InstructionArtifacts::new_constrain(&i64_var));
    all_artifacts.push(InstructionArtifacts::new_range_check(&i64_var, 32));

    // Test shift operations with smaller bit sizes
    // shl truncates variable, so test different sizes
    // Too heavy to test 128 bits, but it just multiplies or divides by 2^rhs
    // Should work the same if div and mul are verified
    all_artifacts.push(InstructionArtifacts::new_shl(&u64_var, &u64_var));
    all_artifacts.push(InstructionArtifacts::new_shr(&u64_var, &u64_var));
    all_artifacts.push(InstructionArtifacts::new_shl(&u32_var, &u32_var));
    all_artifacts.push(InstructionArtifacts::new_shl(&u8_var, &u8_var));
    all_artifacts.push(InstructionArtifacts::new_shr(&u8_var, &u8_var));
    all_artifacts.push(InstructionArtifacts::new_shl(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_shr(&i64_var, &i64_var));
    all_artifacts.push(InstructionArtifacts::new_shl(&i8_var, &i8_var));
    all_artifacts.push(InstructionArtifacts::new_shr(&i8_var, &i8_var));


    // Test binary operations with 32 bits
    all_artifacts.push(InstructionArtifacts::new_xor(&u32_var, &u32_var));
    all_artifacts.push(InstructionArtifacts::new_and(&u32_var, &u32_var));
    all_artifacts.push(InstructionArtifacts::new_or(&u32_var, &u32_var));

    // Test field operations
    all_artifacts.push(InstructionArtifacts::new_add(&field_var, &field_var));
    all_artifacts.push(InstructionArtifacts::new_mul(&field_var, &field_var));
    all_artifacts.push(InstructionArtifacts::new_div(&field_var, &field_var));
    all_artifacts.push(InstructionArtifacts::new_eq(&field_var, &field_var));

    // Field as u32
    all_artifacts.push(InstructionArtifacts::new_truncate(&field_var, 32, 254));

    // u64 as u8
    all_artifacts.push(InstructionArtifacts::new_truncate(&u64_var, 8, 64));

    // i64 as u8
    all_artifacts.push(InstructionArtifacts::new_truncate(&i64_var, 8, 64));

    save_artifacts(all_artifacts, &args.dir);
}
