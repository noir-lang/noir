use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::errors::CliError;

use clap::Args;
use nargo_toml::{PackageSelection, list_cached_git_dependencies};

use super::{LockType, PackageOptions, WorkspaceCommand};

/// Fetch the dependencies of a package from the network
#[derive(Debug, Clone, Args)]
pub(crate) struct FetchCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,
}

impl WorkspaceCommand for FetchCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // The `LockType` lock guards a workspace member's `target/` artifacts, which this command
        // never writes. Git dependencies are downloaded into the global cache during workspace
        // resolution, which serializes concurrent runs with its own `.package-cache` lock before
        // any `LockType` lock would be taken, so `None` is correct here.
        LockType::None
    }
}

pub(crate) fn run(fetched_before: BTreeSet<PathBuf>) -> Result<(), CliError> {
    // Resolving the workspace (which happens before this runs) downloads any missing git
    // dependencies into the global cache. Comparing the cache against the snapshot taken before
    // resolution tells us exactly which dependencies were fetched during this run; ones that were
    // already cached are not reported.
    let fetched_after = list_cached_git_dependencies();
    let newly_fetched: Vec<_> = fetched_after.difference(&fetched_before).collect();

    if newly_fetched.is_empty() {
        return Ok(());
    }

    let count = newly_fetched.len();
    let noun = if count == 1 { "dependency" } else { "dependencies" };
    noirc_errors::println_to_stdout!("Fetched {count} {noun}:");
    for dependency in newly_fetched {
        noirc_errors::println_to_stdout!("  {}", dependency.display());
    }

    Ok(())
}
