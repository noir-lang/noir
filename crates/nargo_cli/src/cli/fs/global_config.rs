use nargo::manifest::{GlobalConfig};
use tracing::debug;


use crate::{constants, errors::FilesystemError};

pub(crate) fn read_global_config_file() -> Option<GlobalConfig> {
    let file_path =
        dirs::home_dir().unwrap().join(constants::NARGO_HOME_FOLDER_NAME).join(constants::NARGO_GLOBAL_CONFIG_FILENAME);

    let read_toml_result = std::fs::read_to_string(&file_path)
    .map_err(|_| FilesystemError::PathNotValid(file_path));
    
    match read_toml_result {
        Ok(toml_as_string) => Some(GlobalConfig::from_toml_str(&toml_as_string).unwrap()),
        Err(err) => {
            debug!("Could not read global config due to {}", err);
            None
        },
    }
}
