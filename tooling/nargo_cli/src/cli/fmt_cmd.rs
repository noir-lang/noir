use std::path::{Path, PathBuf};

use clap::Args;
use nargo_toml::find_package_root;

use crate::errors::CliError;

use super::NargoConfig;

#[derive(Debug, Clone, Args)]
pub(crate) struct FormatCommand {}

pub(crate) fn run(_args: FormatCommand, config: NargoConfig) -> Result<(), CliError> {
    let files = {
        let package = find_package_root(&config.program_dir)?;
        read_files(&package.join("src")).map_err(|error| CliError::Generic(error.to_string()))?
    };

    for file in files {
        let source =
            std::fs::read_to_string(&file).map_err(|error| CliError::Generic(error.to_string()))?;

        let source = nargo_fmt::format(&source);
        std::fs::write(file, source).map_err(|error| CliError::Generic(error.to_string()))?;
    }

    Ok(())
}

fn read_files(path: &Path) -> color_eyre::Result<Vec<PathBuf>> {
    let mut files = vec![];

    if path.is_dir() {
        let entries = std::fs::read_dir(path)?;

        for entry in entries {
            let path = entry?.path();

            if path.is_dir() {
                files.append(&mut read_files(&path)?);
            } else if path.extension().map_or(false, |extension| extension == "nr") {
                files.push(path);
            }
        }
    }

    Ok(files)
}
