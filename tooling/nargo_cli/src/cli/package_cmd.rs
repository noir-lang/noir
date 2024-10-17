use crate::errors::CliError;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs::File;
use std::io::{Seek, SeekFrom};

use super::NargoConfig;
use crate::cli::manifest::profile::Profile;
use crate::cli::manifest::{Manifest, TomlManifest};
// Import File from fm as FmFile
use crate::cli::package::flock::{FileLockGuard, Filesystem};
use crate::cli::package::manifest_normalization::prepare_manifest_for_publish;
use crate::cli::package::source::{list_source_files, MANIFEST_FILE_NAME, ORIGINAL_MANIFEST_FILE_NAME};
use crate::cli::source::SourceId;
// use crate::cli::package::source::list_source_files;
use anyhow::{bail, ensure, Context, Result};
use clap::Args;
use fm::File as FmFile;
use indicatif::{HumanBytes, HumanCount};
use indoc::writedoc;
use nargo::insert_all_files_for_workspace_into_file_manager;
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use std::io::Write;

/// Assemble the local package into a distributable tarball
#[derive(Debug, Clone, Args)]
pub(crate) struct PackageCommand {

}

pub(crate) fn run(args: PackageCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection = PackageSelection::All;
    let selection = default_selection;
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    for package in &workspace {
        let toml_path_utf = Utf8Path::new(toml_path.to_str().unwrap()); // Convert to Utf8Path
        let toml_manifest = TomlManifest::read_from_path(toml_path_utf).unwrap();
        let source_id = SourceId::for_path(toml_path_utf).unwrap();

        let manifest = toml_manifest
            .to_manifest(
                toml_path_utf,
                toml_path_utf,
                source_id,
                Profile::DEV,
                Some(&toml_manifest),
            )
            .with_context(|| format!("failed to parse manifest at: {toml_path_utf}")).unwrap();
        let manifest = Box::new(manifest);
        package_one_impl(package, &workspace, &manifest, toml_path_utf).unwrap();
    }
    Ok(())

}

fn package_one_impl(
    pkg: &Package,
    ws: &Workspace,
    manifest: &Box<Manifest>,
    manifest_path: &Utf8Path
) -> Result<(FileLockGuard)> {

    let recipe = prepare_archive_recipe(pkg, manifest, manifest_path)?;
    let num_files = recipe.len();

    // Package up and test a temporary tarball and only move it to the final location if it actually
    // passes all verification checks. Any previously existing tarball can be assumed as corrupt
    // or invalid, so we can overwrite it if it exists.
    let filename = pkg.name.to_string();
    let target_dir = ws.target_directory_path().join("package");

    let mut dst = Filesystem::new_output_dir(Utf8PathBuf::from(target_dir.to_str().unwrap())).create_rw(format!(".{filename}"), "package scratch space")?;

    dst.set_len(0).with_context(|| format!("failed to truncate: {filename}"))?;

    let uncompressed_size = tar(pkg.name.to_string(), &recipe, &mut dst)?;

    // let mut dst = if opts.verify {
    //     run_verify(pkg, dst, ws, opts.features.clone())
    //         .context("failed to verify package tarball")?
    // } else {
    //     dst
    // };

    dst.seek(SeekFrom::Start(0))?;

    dst.rename(dst.path().with_file_name(filename))?;

    let dst_metadata = dst
        .metadata()
        .with_context(|| format!("failed to stat: {}", dst.path()))?;
    let compressed_size = dst_metadata.len();

    println!(
        "Packaged: {} files, {:.1} ({:.1} compressed)",
        HumanCount(num_files as u64),
        HumanBytes(uncompressed_size),
        HumanBytes(compressed_size),
    );

    Ok(dst)
}

struct ArchiveFile {
    /// The relative path in the archive (not including top-level package name directory).
    path: Utf8PathBuf,
    /// The contents of the file.
    contents: ArchiveFileContents,
}

enum ArchiveFileContents {
    /// Absolute path to the file on disk to add to the archive.
    OnDisk(Utf8PathBuf),

    Generated(Box<dyn Fn() -> Result<Vec<u8>>>),
}

fn prepare_archive_recipe(pkg: &Package, manifest: &Box<Manifest>, manifest_path: &Utf8Path) -> Result<ArchiveRecipe> {
    ensure!(
        pkg.is_library(),
        r"
        cannot archive package `{}` without a `lib` target
        help: add `[lib]` section to package manifest
         --> Nargo.toml
        +   [lib]
        ",
        pkg.name.to_string(),
    );

    let mut recipe = source_files(pkg)?;

    // Sort the recipe before any checks, to ensure generated errors are reproducible.
    sort_recipe(&mut recipe);

    check_filenames(&recipe)?;
    //todo add later
    // check_no_reserved_files(&recipe)?;

    recipe.push(ArchiveFile {
        path: MANIFEST_FILE_NAME.into(),
        contents: ArchiveFileContents::Generated({
            let pkg_clone = pkg.clone();
            let manifest_clone = manifest.clone();
            Box::new(move || {
                normalize_manifest(pkg_clone.clone(), manifest_clone.clone())
            })
        }),
    });;

    // Add original manifest file.
    recipe.push(ArchiveFile {
        path: ORIGINAL_MANIFEST_FILE_NAME.into(),
        contents: ArchiveFileContents::OnDisk(manifest_path.to_path_buf()),
    });

    // // Add README file
    // if let Some(readme) = &pkg.manifest.metadata.readme {
    //     recipe.push(ArchiveFile {
    //         path: DEFAULT_README_FILE_NAME.into(),
    //         contents: ArchiveFileContents::OnDisk(readme.clone()),
    //     })
    // }
    //
    // // Add LICENSE file
    // if let Some(license) = &pkg.manifest.metadata.license_file {
    //     recipe.push(ArchiveFile {
    //         path: DEFAULT_LICENSE_FILE_NAME.into(),
    //         contents: ArchiveFileContents::OnDisk(license.clone()),
    //     })
    // }
    //
    // // Add archive version file.
    // recipe.push(ArchiveFile {
    //     path: VERSION_FILE_NAME.into(),
    //     contents: ArchiveFileContents::Generated(Box::new(|| Ok(VERSION.to_string().into_bytes()))),
    // });
    //
    // // Add VCS info file.
    // if let Ok(repo) = PackageRepository::open(pkg) {
    //     if let Some(vcs_info) = extract_vcs_info(repo, opts)? {
    //         recipe.push(ArchiveFile {
    //             path: VCS_INFO_FILE_NAME.into(),
    //             contents: ArchiveFileContents::Generated({
    //                 Box::new(move || Ok(serde_json::to_string(&vcs_info)?.into_bytes()))
    //             }),
    //         });
    //     }
    // };

    // Put generated files in right order within the recipe.
    sort_recipe(&mut recipe);

    // Assert there are no duplicates. We make use of the fact, that recipe is now sorted.
    assert!(
        recipe.windows(2).all(|w| w[0].path != w[1].path),
        "duplicate files in package recipe: {duplicates}",
        duplicates = recipe
            .windows(2)
            .filter(|w| w[0].path == w[1].path)
            .map(|w| w[0].path.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(recipe)
}

/// A listing of files to include in the archive, without actually building it yet.
///
/// This struct is used to facilitate both building the package, and listing its contents without
/// actually making it.
type ArchiveRecipe = Vec<ArchiveFile>;


fn source_files(pkg: &Package) -> Result<ArchiveRecipe> {
    list_source_files(pkg)?
        .into_iter()
        .map(|on_disk| {
            let path = on_disk.strip_prefix(&pkg.root_dir)?.to_owned();
            Ok(ArchiveFile {
                path,
                contents: ArchiveFileContents::OnDisk(on_disk),
            })
        })
        .collect()
}
const VERSION_FILE_NAME: &str = "VERSION";


/// Sort archive files alphabetically, putting the version file first.
fn sort_recipe(recipe: &mut ArchiveRecipe) {
    recipe.sort_unstable_by_key(|f| {
        let priority = if f.path == VERSION_FILE_NAME { 0 } else { 1 };
        (priority, f.path.clone())
    });
}

fn check_filenames(recipe: &ArchiveRecipe) -> Result<()> {
    for ArchiveFile { path, .. } in recipe {
        const BAD_CHARS: &[char] = &['/', '\\', '<', '>', ':', '"', '|', '?', '*'];
        for component in path.components() {
            let name = component.as_str();
            if let Some(c) = BAD_CHARS.iter().find(|&&c| name.contains(c)) {
                bail!("cannot package a filename with a special character `{c}`: {path}");
            }
        }

        //todo add this later
        // if restricted_names::is_windows_restricted_path(path.as_std_path()) {
        //     bail!("cannot package file `{path}`, it is a Windows reserved filename");
        // }
    }
    Ok(())
}

#[tracing::instrument(level = "trace", skip_all)]
fn normalize_manifest(pkg: Package, manifest: Box<Manifest>) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    writedoc!(
        &mut buf,
        r##"
        # Code generated by scarb package -p {package_name}; DO NOT EDIT.
        #
        # When uploading packages to the registry Scarb will automatically
        # "normalize" {toml} files for maximal compatibility
        # with all versions of Scarb and also rewrite `path` dependencies
        # to registry dependencies.
        #
        # If you are reading this file be aware that the original {toml}
        # will likely look very different (and much more reasonable).
        # See {orig} for the original contents.
        "##,
        package_name = pkg.name,
        toml = MANIFEST_FILE_NAME,
        orig = ORIGINAL_MANIFEST_FILE_NAME,
    )?;
    writeln!(&mut buf)?;

    let manifest = prepare_manifest_for_publish(manifest)?;
    let toml = toml::to_string_pretty(&manifest)?;
    writeln!(&mut buf, "{toml}")?;

    Ok(buf)
}



/// Compress and package the recipe, and write it into the given file.
///
/// Returns the uncompressed size of the contents of the archive.
fn tar(
    pkg_id: String,
    recipe: &ArchiveRecipe,
    dst: &mut File,
) -> Result<u64> {
    const COMPRESSION_LEVEL: i32 = 22;
    let encoder = zstd::stream::Encoder::new(dst, COMPRESSION_LEVEL)?;
    let mut ar = tar::Builder::new(encoder);

    let base_path = Utf8PathBuf::from(pkg_id);

    let mut uncompressed_size = 0;
    for ArchiveFile { path, contents } in recipe {
        // Now you can access the fields of `file` directly
        // ws.config()
        //     .ui()
        //     .verbose(Status::new("Archiving", path.as_str()));

        let archive_path = base_path.join(path);
        let mut header = tar::Header::new_gnu();
        match contents {
            ArchiveFileContents::OnDisk(disk_path) => {
                let mut file = File::open(&disk_path)
                    .with_context(|| format!("failed to open for archiving: {disk_path}"))?;

                let metadata = file
                    .metadata()
                    .with_context(|| format!("failed to stat: {disk_path}"))?;

                header.set_metadata_in_mode(&metadata, tar::HeaderMode::Deterministic);

                // Although the `set_metadata_in_mode` call above should set `mtime` to a
                // deterministic value, it fails to do so due to
                // https://github.com/alexcrichton/tar-rs/issues/341.
                // Also, the constant value used there is funky and I do not feel convinced about
                // its stability. Therefore, we use our own `mtime` value explicitly here.
                //
                // From `set_metadata_in_mode` implementation in `tar` crate:
                // > We could in theory set the mtime to zero here, but not all
                // > tools seem to behave well when ingesting files with a 0
                // > timestamp.
                header.set_mtime(1);

                header.set_cksum();

                ar.append_data(&mut header, &archive_path, &mut file)
                    .with_context(|| format!("could not archive source file: {disk_path}"))?;

                uncompressed_size += metadata.len();
            }

            ArchiveFileContents::Generated(generator) => {
                // Zamieniamy `generator` na `mut generator` aby go przenieść i wywołać
                let mut generator = generator;
                let contents = generator()?;

                header.set_entry_type(tar::EntryType::file());
                header.set_mode(0o644);
                header.set_size(contents.len() as u64);

                // Same as above.
                header.set_mtime(1);

                header.set_cksum();

                ar.append_data(&mut header, &archive_path, contents.as_slice())
                    .with_context(|| format!("could not archive source file: {path}"))?;

                uncompressed_size += contents.len() as u64;
            }
        }
    }

    let encoder = ar.into_inner()?;
    encoder.finish()?;
    Ok(uncompressed_size)
}
