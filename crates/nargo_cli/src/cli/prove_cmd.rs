use acvm::Backend;
use clap::Args;
// use nargo::artifacts::program::PreprocessedProgram;
// use nargo::ops::{preprocess_program, prove_execution, verify_proof};
// use noirc_abi::input_parser::Format;


use super::NargoConfig;
// use super::{
//     compile_cmd::compile_circuit,
//     fs::{
//         common_reference_string::{
//             read_cached_common_reference_string, update_common_reference_string,
//             write_cached_common_reference_string,
//         },
//         inputs::{read_inputs_from_file, write_inputs_to_file},
//         program::read_program_from_file,
//         proof::save_proof_to_dir,
//     },
// };
use crate::{
    // cli::execute_cmd::execute_program,
    constants::{PROOFS_DIR, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    /// The name of the proof
    proof_name: Option<String>,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    /// Verify proof after proving
    #[arg(short, long)]
    verify: bool,

    // #[clap(flatten)]
    // compile_options: CompileOptions,

    /// Argument or environment variable  to specify path to backend executable, default `$USER/.nargo/bin/bb.js`
    #[arg(long, env)]
    backend_executable: Option<String>,

    #[arg(long, env)]
    recursive: Option<bool>,
    
    #[clap(raw=true)]
    others: Option<String>,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: ProveCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    use tracing::{info, debug};
    // use tracing_subscriber;
    use std::process::{Command, Stdio};
    use std::io::{BufRead, BufReader};
    use which::which;

    tracing_subscriber::fmt::init();

    let backend_executable_path = if let Some(backend_executable) = args.backend_executable {
        debug!("Backend path specified as argument or environment variable `{}`", backend_executable);
        backend_executable        
    } else { 
        match which("bb.js") {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => {
                let home_dir = dirs::home_dir().unwrap().join(".nargo").join("backends").join("bb.js");
                debug!("bb.js not found on path, choosing default `{}`", home_dir.to_string_lossy());
                home_dir.to_string_lossy().to_string()
            },
        }
    };

    let mut bb_args = vec!["prove", "-v"];
    if let Some(is_recursive) = args.recursive {
        if is_recursive {
            debug!("Is recursive `{}`", is_recursive);
            bb_args.push("--recursive");
        }
    }
    debug!("About to spawn new command `{}`", backend_executable_path);
    let mut backend = Command::new(backend_executable_path.to_owned())
    .args(bb_args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
        .spawn().expect(format!("Failed to execute backend with `{}`, specify with `--backend-executable` argument", backend_executable_path).as_str());

    let stdout = backend.stderr.take().expect("no stdout");
    let result = BufReader::new(stdout)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));

    Ok(())
}

