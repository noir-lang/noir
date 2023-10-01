use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Cannot read file {0} - {1}")]
    ReadFailed(PathBuf, std::io::Error),

    #[error("noirfmt.toml is badly formed, could not parse.\n\n {0}")]
    MalformedFile(#[from] toml::de::Error),
}
