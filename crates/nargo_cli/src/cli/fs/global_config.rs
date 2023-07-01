use nargo::manifest::GlobalConfig;
use tracing::debug;

use crate::{constants, errors::FilesystemError};
use nargo::manifest::errors::GlobalConfigError;

pub(crate) fn read_global_config_file() -> Option<GlobalConfig> {
    let file_path = dirs::home_dir()
        .unwrap()
        .join(constants::NARGO_HOME_FOLDER_NAME)
        .join(constants::NARGO_GLOBAL_CONFIG_FILENAME);

    let read_toml_result =
        std::fs::read_to_string(&file_path).map_err(|_| FilesystemError::PathNotValid(file_path));

    match read_toml_result {
        Ok(toml_as_string) => Some(GlobalConfig::from_toml_str(&toml_as_string).unwrap()),
        Err(err) => {
            debug!("Could not read global config due to {}", err);
            None
        }
    }
}

pub(crate) fn write_global_config_file(
    global_config: GlobalConfig,
) -> Result<(), GlobalConfigError> {
    let file_path = dirs::home_dir()
        .unwrap()
        .join(constants::NARGO_HOME_FOLDER_NAME)
        .join(constants::NARGO_GLOBAL_CONFIG_FILENAME);

    match global_config.to_toml_str() {
        Ok(toml_string) => std::fs::write(file_path, toml_string)
            .map_err(|err| -> GlobalConfigError { GlobalConfigError::Generic(err.to_string()) }),
        Err(err) => Err(err),
    }
}
