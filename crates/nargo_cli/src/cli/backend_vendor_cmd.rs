use color_eyre::ErrorKind;
use tracing::{debug};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use which::which;

use clap::Args;

use crate::errors::CliError;

use super::NargoConfig;

#[derive(Debug, Clone, Args)]
pub(crate) struct BackendSubcommand {
    /// Argument or environment variable  to specify path to backend executable, default `$USER/.nargo/bin/bb.js`
    #[arg(long, env)]
    pub(crate) backend_executable: PathBuf,

    // Thise option should allow for -- --args1 .. --argsN to pass to backend
    #[clap(last=true)]
    pub(crate) raw_pass_through: Option<Vec<String>>,
    
    #[arg(long, env, default_value="bb.js", hide=true)]
    pub(crate) default_backend: PathBuf,
}

pub(crate) fn resolve_backend<'a>(args: &'a BackendSubcommand, config: &'a NargoConfig) -> Result<PathBuf, CliError>  {
    match which(args.backend_executable.clone()) {
        Ok(be_path) => Ok(be_path),
        Err(_) => {
            debug!("Neither the `--backend_executable` argument nor the `$BACKEND_EXECUTABLE` environment variable is set to specify the path for the backend vendor.");
            match which(args.default_backend.clone()) {
                Ok(db_path) => Ok(db_path),
                Err(_) => {
                    debug!("Neither the `--default_backend` argument nor the `$DEFAULT_BACKEND` environment variable is set to specify the path for the backend vendor.");
                    let assummed_default_path = dirs::home_dir().unwrap().join(".nargo").join("backends").join("bin").join("bb.js");
                    match which(&assummed_default_path) {
                        Ok(ad_path) => Ok(ad_path),
                        Err(_) => {
                            debug!("The assumed default path '{:?}' does not contain a valid executable. Please verify that your `Nargo` program is correctly installed.", assummed_default_path);
                            Err(CliError::Generic("Could not find suitable backend vendor to execute command.".to_string()))
                        }
                    }
        
                }
            }
        },
    }

}

pub(crate) fn execute_backend_cmd(backend_executable_path: &PathBuf, backend_args: Vec<String>) -> Result<i32, CliError> {
    debug!("About to spawn new command `{:?} {}`", backend_executable_path, backend_args.join(" "));
    let mut backend = Command::new(backend_executable_path.to_owned())
    .args(backend_args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
        .spawn().expect(format!("Failed to execute backend with `{:?}`, specify with `--backend-executable` argument", backend_executable_path).as_str());

    let stderr = backend.stderr.take().expect("no stderr");
    BufReader::new(stderr)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));

    let stdout = backend.stdout.take().expect("no stdout");
    BufReader::new(stdout)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));
    
    let exit_code = backend.wait().unwrap().code().expect("Expected exit code.");
    debug!("Backend vendor exit code: {exit_code}");

    Ok(exit_code)
}

// /// Create proof for this program. The proof is returned as a hex encoded string.
// #[derive(Debug, Clone, Args)]
// pub(crate) struct BackendVendorCommand {
//     /// The name of the proof
//     proof_name: Option<String>,

//     /// The name of the circuit build files (ACIR, proving and verification keys)
//     circuit_name: Option<String>,

//     /// Verify proof after proving
//     #[arg(short, long)]
//     verify: bool,

//     // #[clap(flatten)]
//     // compile_options: CompileOptions,

//     /// Argument or environment variable  to specify path to backend executable, default `$USER/.nargo/bin/bb.js`
//     #[arg(long, env)]
//     backend_executable: Option<String>,

//     #[arg(long, env)]
//     recursive: Option<bool>,
    
//     // Thise option should allow for -- --args to pass to backend
//     #[clap(last=true)]
//     raw_pass_through: Option<Vec<String>>,
// }

// pub(crate) fn run(
//     args: ProveCommand,
//     _config: NargoConfig,
// ) -> Result<(), CliError> {

//     debug!("{}", args::name);
//     debug!("Args: {:?}", args);
//     debug!("Cfg: {:?}", _config);

//     let backend_executable_path = if let Some(backend_executable) = args.backend_executable {
//         debug!("Backend path specified as argument or environment variable `{}`", backend_executable);
//         backend_executable        
//     } else { 
//         match which("bb.js") {
//             Ok(path) => path.to_string_lossy().to_string(),
//             Err(_) => {
//                 let home_dir = dirs::home_dir().unwrap().join(".nargo").join("backends").join("bb.js");
//                 debug!("bb.js not found on path, choosing default `{}`", home_dir.to_string_lossy());
//                 home_dir.to_string_lossy().to_string()
//             },
//         }
//     };

//     let mut raw_pass_through= args.raw_pass_through.unwrap_or_default();
//     let mut backend_args = vec!["prove".to_string()];
//     backend_args.append(&mut raw_pass_through);

//     debug!("About to spawn new command `{} {}`", backend_executable_path, backend_args.join(" "));
//     let mut backend = Command::new(backend_executable_path.to_owned())
//     .args(backend_args)
//     .stdout(Stdio::piped())
//     .stderr(Stdio::piped())
//         .spawn().expect(format!("Failed to execute backend with `{}`, specify with `--backend-executable` argument", backend_executable_path).as_str());

//     let stderr = backend.stderr.take().expect("no stderr");
//     BufReader::new(stderr)
//         .lines()
//         .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));

//     let stdout = backend.stdout.take().expect("no stdout");
//     BufReader::new(stdout)
//         .lines()
//         .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));
    
//     Ok(())
// }

