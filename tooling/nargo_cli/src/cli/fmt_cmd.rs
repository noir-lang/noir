use std::path::{Path, PathBuf};

use clap::Args;
use fm::FileManager;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::def_map::parse_file;

use crate::errors::CliError;

use super::NargoConfig;

#[derive(Debug, Clone, Args)]
pub(crate) struct FormatCommand {}

pub(crate) fn run(_args: FormatCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(&toml_path, PackageSelection::All)?;

    for package in &workspace {
        let files = {
            read_files(&package.root_dir.join("src"))
                .map_err(|error| CliError::Generic(error.to_string()))?
        };

        let mut file_manager = FileManager::new(&package.root_dir);
        for file in files {
            let file_id = file_manager.add_file(&file).expect("file exists");
            let (parsed_module, errors) = parse_file(&file_manager, file_id);

            if !errors.is_empty() {
                let errors = errors
                    .into_iter()
                    .map(|error| {
                        let error: CustomDiagnostic = error.into();
                        error.in_file(file_id)
                    })
                    .collect();
                let _ = super::compile_cmd::report_errors::<()>(Err(errors), &file_manager, false);
                continue;
            }

            let source =
                nargo_fmt::format(file_manager.fetch_file(file_id).source(), parsed_module);
            std::fs::write(file, source).map_err(|error| CliError::Generic(error.to_string()))?;
        }
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
