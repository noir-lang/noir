use std::str::FromStr;

use anyhow::{Context, Result};
use camino::Utf8Path;
use toml_edit::DocumentMut;

use crate::cli::internal::fsx;
pub use dep_id::DepId;
pub use dep_type::{DepType, SectionArgs};

pub mod add;
mod dep_id;
mod dep_type;
mod remove;
mod tomlx;

pub trait Op {
    fn apply_to(self: Box<Self>, doc: &mut DocumentMut, ctx: OpCtx<'_>) -> Result<()>;
}

pub struct OpCtx<'c> {
    pub manifest_path: &'c Utf8Path,
    pub opts: &'c EditManifestOptions,
}

//todo fix for azted if needed
// pub struct EditManifestOptions<'c> {
pub struct EditManifestOptions {
    pub dry_run: bool,
}

/// Edit manifest file (for example, add dependencies).
#[tracing::instrument(level = "trace", skip(ops, opts))]
pub fn edit(
    manifest_path: &Utf8Path,
    ops: Vec<Box<dyn Op>>,
    opts: EditManifestOptions,
) -> Result<()> {
    let manifest_path = fsx::canonicalize_utf8(manifest_path)?;

    let original_raw_manifest = fsx::read_to_string(&manifest_path)?;
    let mut doc = DocumentMut::from_str(&original_raw_manifest)
        .with_context(|| format!("failed to read manifest at: {manifest_path}"))?;

    for op in ops {
        op.apply_to(
            &mut doc,
            OpCtx {
                manifest_path: &manifest_path,
                opts: &opts,
            },
        )?;
    }

    // TODO(#128): Sort dependencies and scripts etc.

    let new_raw_manifest = doc.to_string();
    if new_raw_manifest == original_raw_manifest {
        println!("no changes have to be made");
    } else if opts.dry_run {
        println!("aborting due to dry run");
    } else {
        fsx::write(manifest_path, new_raw_manifest.as_bytes())?;
    }

    Ok(())
}
