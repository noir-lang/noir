use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::debug;
use which::which;

use clap::Args;

use crate::constants;
use crate::errors::CliError;

use super::NargoConfig;

#[derive(Debug, Clone, Args)]
pub(crate) struct BackendCommand {

    /// Argument or environment variable  to specify path to backend executable, default `$USER/.nargo/backends/bin/bb.js`
    #[arg(long, env)]
    pub(crate) backend_executable: PathBuf,

    #[arg(long, env, default_value = "bb.js", hide = true)]
    pub(crate) default_backend: PathBuf,

    /// Pass `-- --args1 .. --argsN` to backend
    #[clap(last = true)]
    pub(crate) backend_arguments: Option<Vec<String>>,

}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProofArtifact {

    #[arg(env, long, hide = true)]
    pub(crate) nargo_default_proof_dir: Option<PathBuf>,

    #[arg(env, long, hide = true)]
    pub(crate) nargo_default_proof_name: Option<String>,

    /// ACIR file desired location path
    #[arg(env, long)]
    pub(crate) nargo_proof_path: Option<PathBuf>,

    #[arg(env, long)]
    pub(crate) nargo_verification_key_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WitnessArtifact {
    /// Witness file desired location path
    #[arg(env, long)]
    pub(crate) nargo_witness_path: Option<PathBuf>,

}

pub(crate) fn resolve_backend<'a>(
    args: &'a BackendCommand,
    config: &'a NargoConfig,
) -> Result<PathBuf, CliError> {
    match which(args.backend_executable.clone()) {
        Ok(be_path) => Ok(be_path),
        Err(_) => {
            debug!("Neither the `--backend_executable` argument nor the `$BACKEND_EXECUTABLE` environment variable pointed to a valid backend vendor.");
            match which(args.default_backend.clone()) {
                Ok(db_path) => Ok(db_path),
                Err(_) => {
                    debug!("Neither the `--default_backend` argument nor the `$DEFAULT_BACKEND` environment variable pointed to a valid backend vendor.");
                    let assummed_default_path = dirs::home_dir()
                        .unwrap()
                        .join(".nargo")
                        .join("backends")
                        .join("bin")
                        .join("bb.js");
                    match which(&assummed_default_path) {
                        Ok(ad_path) => Ok(ad_path),
                        Err(_) => {
                            debug!("The assumed default path '{:?}' does not contain a valid executable. Please verify that your `Nargo` program is correctly installed.", assummed_default_path);
                            Err(CliError::Generic(
                                "Could not find suitable backend vendor to execute command."
                                    .to_string(),
                            ))
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn execute_backend_cmd(
    backend_executable_path: &PathBuf,
    backend_args: Vec<String>,
    project_dir: &PathBuf,
    envs: Option<HashMap<String, String>>,
) -> Result<i32, CliError> {
    
    debug!("Command about to spawn: `{:?} {}`", backend_executable_path, backend_args.join(" "));
    debug!("Command Current Working Directory $cwd: {:?}", project_dir);
    debug!("Command environment $env: {:?}", envs);

    let mut backend = Command::new(backend_executable_path.to_owned());
    backend
    .args(backend_args)
    .current_dir(project_dir)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    if let Some(env) = envs {
        backend.envs(env);
    };

    let mut child_process = backend.spawn().expect(format!("Failed to execute backend with `{:?}`, specify with `--backend-executable` argument", backend_executable_path).as_str());

    let stderr = child_process.stderr.take().expect("no stderr");
    BufReader::new(stderr)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));

    let stdout = child_process.stdout.take().expect("no stdout");
    BufReader::new(stdout)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));

    let exit_code = child_process.wait().unwrap().code().expect("Expected exit code. Process received SIGKILL?");
    
    debug!("Backend vendor exit code: {exit_code}");

    Ok(exit_code)
}

pub(crate) fn configure_proof_artifact(config: &NargoConfig, proof_options: &mut ProofArtifact) {
    // let nargo_package_root = ;
    
    proof_options.nargo_default_proof_dir = Some(proof_options.nargo_default_proof_dir.clone().unwrap_or_else(|| {
        let mut target = config.nargo_package_root.clone();
        target.push(constants::PROOFS_DIR);
        target

    }));
    let nargo_artifact_name = config.nargo_artifact_name.as_ref().unwrap().clone();
    proof_options.nargo_default_proof_name = Some(proof_options.nargo_default_proof_name.clone().unwrap_or_else(|| {
        nargo_artifact_name
    }));

    let nargo_default_proof_dir = proof_options.nargo_default_proof_dir.as_ref().unwrap();
    let nargo_default_proof_name = proof_options.nargo_default_proof_name.as_ref().unwrap();

    proof_options.nargo_proof_path = Some(proof_options.nargo_proof_path.clone().unwrap_or_else(|| {
        let mut target = nargo_default_proof_dir.clone();
        let mut nargo_proof_path = nargo_default_proof_name.clone();
        nargo_proof_path.push_str(".");
        nargo_proof_path.push_str(constants::PROOF_EXT);
        target.push(nargo_proof_path);
        target

    }));
    proof_options.nargo_verification_key_path = Some(proof_options.nargo_verification_key_path.clone().unwrap_or_else(|| {
        let mut target = nargo_default_proof_dir.clone();
        let mut nargo_proof_path = nargo_default_proof_name.clone();
        nargo_proof_path.push_str(".");
        nargo_proof_path.push_str(constants::VERIFICATION_KEY_EXT);
        target.push(nargo_proof_path);
        target

    }));

}

pub(crate) fn configure_witness_artifact(config: &NargoConfig, witness_options: &mut WitnessArtifact) {
    witness_options.nargo_witness_path = Some(witness_options.nargo_witness_path.clone().unwrap_or_else(|| {
        let mut target = config.nargo_target_dir.as_ref().unwrap().clone();
        let mut nargo_witness_name = config.nargo_artifact_name.as_ref().unwrap().clone();
        nargo_witness_name.push_str(".");
        nargo_witness_name.push_str(constants::WITNESS_EXT);
        target.push(nargo_witness_name);
        target

    }));
}

pub(crate) fn run(
    backend_subcommand: &str,
    args: BackendCommand,
    config: NargoConfig,
) -> Result<i32, CliError> {    

    // configure_proof_artifact(&config, &mut args);

    debug!("Supplied Prove arguments: {:?}", args);

    let backend_executable_path = resolve_backend(&args, &config)?;
    let mut raw_pass_through= args.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(backend_subcommand)];
    backend_args.append(&mut raw_pass_through);

    let exit_code = execute_backend_cmd(&backend_executable_path, backend_args, &config.nargo_package_root, Option::None);

    match exit_code {
        Ok(code) => {
            if code > 0 {
                Err(CliError::Generic(format!("Backend exited with failure code: {}", code)))
            } else {
                Ok(code)
            }
        },
        Err(err) => Err(err),
    }
    
}

