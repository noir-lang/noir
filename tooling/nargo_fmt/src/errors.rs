use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Cannot read file {0} - {1}")]
    ReadFailed(PathBuf, std::io::Error),

    #[error("{0} is badly formed, could not parse.\n\n {1}")]
    MalformedFile(PathBuf, toml::de::Error),
}
