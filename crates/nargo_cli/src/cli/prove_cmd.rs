use acvm::Backend;
use clap::Args;

use super::NargoConfig;
use crate::{
    // constants::{PROOFS_DIR, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
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
    
    // Thise option should allow for -- --args to pass to backend
    #[clap(last=true)]
    raw_pass_through: Option<Vec<String>>,
}

pub(crate) fn run<B: Backend>(
    _: &B,
    args: ProveCommand,
    _: NargoConfig,
) -> Result<(), CliError<B>> {
    use tracing::{debug};
    use std::process::{Command, Stdio};
    use std::io::{BufRead, BufReader};
    use which::which;

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

    let mut raw_pass_through= args.raw_pass_through.unwrap();
    let mut backend_args = vec!["prove".to_string()];
    backend_args.append(&mut raw_pass_through);

    debug!("About to spawn new command `{} {}`", backend_executable_path, backend_args.join(" "));
    let mut backend = Command::new(backend_executable_path.to_owned())
    .args(backend_args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
        .spawn().expect(format!("Failed to execute backend with `{}`, specify with `--backend-executable` argument", backend_executable_path).as_str());

    let stderr = backend.stderr.take().expect("no stderr");
    BufReader::new(stderr)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));

    let stdout = backend.stdout.take().expect("no stdout");
    BufReader::new(stdout)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));
    
    Ok(())
}

