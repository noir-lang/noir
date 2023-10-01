use std::{fs::DirEntry, path::Path};

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

    let config = nargo_fmt::Config::read(&config.program_dir)
        .map_err(|err| CliError::Generic(err.to_string()))?;

    for package in &workspace {
        let mut file_manager = FileManager::new(&package.root_dir);

        visit_noir_files(&package.root_dir.join("src"), &mut |entry| {
            let file_id = file_manager.add_file(&entry.path()).expect("file exists");
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
                return Ok(());
            }

            let source = nargo_fmt::format(
                file_manager.fetch_file(file_id).source(),
                parsed_module,
                &config,
            );

            std::fs::write(entry.path(), source)
        })
        .map_err(|error| CliError::Generic(error.to_string()))?;
    }
    Ok(())
}

fn visit_noir_files(
    dir: &Path,
    cb: &mut dyn FnMut(&DirEntry) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_noir_files(&path, cb)?;
            } else if entry.path().extension().map_or(false, |extension| extension == "nr") {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}
