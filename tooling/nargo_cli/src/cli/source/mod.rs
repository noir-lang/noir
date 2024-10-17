use anyhow::Result;
use async_trait::async_trait;

pub use id::*;
use nargo::package::Package;
use crate::cli::manifest::{ManifestDependency, Summary};
use crate::cli::package::id::PackageId;

mod id;
mod scarb_stable_hash;
pub mod canonical_url;

/// Something that finds and downloads remote packages based on names and versions.
#[async_trait]
pub trait Source {
    /// Attempts to find the packages that match a dependency request.
    async fn query(&self, dependency: &ManifestDependency) -> Result<Vec<Summary>>;

    /// Fetches the full package for each name and version specified.
    async fn download(&self, id: PackageId) -> Result<Package>;
}
