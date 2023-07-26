use crate::cli::arguments::ContractArtifact;
use crate::errors::{BackendVendorError, CliError};
use crate::{constants};
use acvm::Backend;
use clap::Args;
use dirs::home_dir;
use nameof::name_of;
use nargo::manifest::GlobalConfig;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf};
use std::process::{Command, Stdio};
use tracing::debug;
use which::which;

use super::arguments::NargoConfig;
use super::fs::global_config;


fn default_backend_path() -> PathBuf {
    let backend_assumed_path_buf = home_dir()
        .unwrap()
        .join(constants::NARGO_HOME_FOLDER_NAME)
        .join(constants::NARGO_BACKENDS_FOLDER_NAME)
        .join("bin")
        .join("bb.js");

    let som = match global_config::read_global_config_file() {
        Some(gc) => match gc.backends {
            Some(be) => be.default,
            None => None,
        },
        None => None,
    };

    match som {
        Some(dcf) => PathBuf::from(dcf),
        None => backend_assumed_path_buf,
    }
}
#[derive(Debug, Clone, Args)]
pub(crate) struct BackendOptions {
    /// Argument or environment variable to specify path to backend executable
    #[arg(env, long, default_value_os_t = default_backend_path())]
    pub(crate) backend_executable: PathBuf,

    /// Pass arguments to the backend behind `--`, eg. nargo backend -- prove -ex-flag
    #[clap(last = true)]
    pub(crate) backend_arguments: Option<Vec<String>>,
}


pub(crate) fn resolve_backend<'a>(args: &'a BackendOptions) -> Result<PathBuf, BackendVendorError> {
    match which(&args.backend_executable) {
            Ok(be_path) => Ok(be_path),
            Err(_) => {
                Err(BackendVendorError::Generic(format!("{:?} does not point to valid backend. Override with `--backend_executable` argument or the `$BACKEND_EXECUTABLE` environment variable pointing to a valid backend vendor.", args.backend_executable)))
            },
        }
}

pub(crate) fn execute_backend_cmd(
    backend_executable_path: &PathBuf,
    backend_args: Vec<String>,
    config: &NargoConfig,
) -> Result<(), BackendVendorError> {
    debug!("Command about to spawn: `{:?} {}`", backend_executable_path, backend_args.join(" "));
    debug!("Command Current Working Directory $cwd: {:?}", config.nargo_package_root);

    let mut envs = HashMap::new();
    envs.insert(
        name_of!(nargo_artifact_path in NargoConfig).to_uppercase(),
        String::from(config.nargo_artifact_path.clone().unwrap().as_os_str().to_str().unwrap()),
    );
    envs.insert(
        name_of!(nargo_witness_path in NargoConfig).to_uppercase(),
        String::from(config.nargo_witness_path.clone().unwrap().as_os_str().to_str().unwrap()),
    );
    // envs.insert(
    //     name_of!(nargo_proof_path in NargoConfig).to_uppercase(),
    //     String::from(config.nargo_proof_path.clone().as_os_str().to_str().unwrap()),
    // );
    envs.insert(
        name_of!(nargo_verification_key_path in NargoConfig).to_uppercase(),
        String::from(
            config.nargo_verification_key_path.clone().unwrap().as_os_str().to_str().unwrap(),
        ),
    );
    envs.insert(
        name_of!(nargo_contract_path in ContractArtifact).to_uppercase(),
        String::from(config.nargo_contract_path.clone().unwrap().as_os_str().to_str().unwrap()),
    );
    debug!("Command environment $env: {:?}", envs);

    let mut backend = Command::new(backend_executable_path.to_owned());
    backend
        .args(backend_args)
        .current_dir(config.nargo_package_root.clone())
        .envs(envs)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child_process = backend.spawn().expect(
        format!(
            "Failed to execute backend with `{:?}`, specify with `--backend-executable` argument",
            backend_executable_path
        )
        .as_str(),
    );

    let stderr = child_process.stderr.take().expect("no stderr");
    BufReader::new(stderr)
        .lines()
        .for_each(|line| println!("{}", line.unwrap_or_default().to_string()));

    let stdout = child_process.stdout.take().expect("no stdout");
    BufReader::new(stdout)
        .lines()
        .for_each(|line| debug!("{}", line.unwrap_or_default().to_string()));

    match child_process.wait() {
        Ok(exit_status) => {
            if exit_status.success() {
                Ok(())
            } else {
                Err(BackendVendorError::Generic(format!(
                    "Backend exited with failure code: {}",
                    exit_status.code().unwrap()
                )))
            }
        }
        Err(err) => Err(BackendVendorError::Generic(err.to_string())),
    }
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    args: BackendOptions,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    debug!("Supplied Prove arguments: {:?}", args);

    let backend_executable_path = resolve_backend(&args)?;
    let raw_pass_through = args.backend_arguments.unwrap_or_default();
    execute_backend_cmd(&backend_executable_path, raw_pass_through, &config)
        .map_err(|e| CliError::BackendVendorError(e))
}

pub(crate) fn set_default_paths(config: &mut NargoConfig) {
    // We default a nargo_artifact_name to parent folder name
    // config.nargo_artifact_name = Some(config.nargo_artifact_name.clone().unwrap_or_else(|| {
    //     // String::from("main")
    //     config
    //         .nargo_package_root
    //         .components()
    //         .last()
    //         .unwrap()
    //         .as_os_str()
    //         .to_string_lossy()
    //         .to_string()
    // }));
    // We default a NARGO_TARGET_DIR to NARGO_PACKAGE_ROOT + `/target`
    config.nargo_target_dir = Some(config.nargo_target_dir.clone().unwrap_or_else(|| {
        let mut target = config.nargo_package_root.clone();
        target.push(constants::TARGET_DIR);
        target
    }));
    // We default each of below items to `NARGO_PACKAGE_ROOT` + `/target/` + `NARGO_ARTIFACT_NAME` + `file extension`
    config.nargo_artifact_path =
        derive_default_path(&config, config.nargo_artifact_path.clone(), constants::ACIR_EXT);
    config.nargo_witness_path =
        derive_default_path(&config, config.nargo_witness_path.clone(), constants::WITNESS_EXT);
    // config.nargo_proof_path =
        // derive_default_path(&config, config.nargo_proof_path.clone(), constants::PROOF_EXT);
    config.nargo_verification_key_path = derive_default_path(
        &config,
        config.nargo_verification_key_path.clone(),
        constants::VERIFICATION_KEY_EXT,
    );
    config.nargo_contract_path =
        derive_default_path(&config, config.nargo_contract_path.clone(), constants::CONTRACT_EXT);
}

fn derive_default_path(
    config: &NargoConfig,
    current_path: Option<PathBuf>,
    ext: &str,
) -> Option<PathBuf> {
    Some(current_path.clone().unwrap_or_else(|| {
        let mut target = config.nargo_target_dir.as_ref().unwrap().clone();
        let mut nargo_witness_name = config.nargo_artifact_name.clone();
        nargo_witness_name.push('.');
        nargo_witness_name.push_str(ext);
        target.push(nargo_witness_name);
        target
    }))
}
