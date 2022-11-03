use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use acvm::Language;
use noirc_driver::Driver;
use noirc_frontend::graph::{CrateId, CrateType};

use crate::{
    errors::CliError,
    toml::{Config, Dependency},
};

/// Creates a unique folder name for a GitHub repo
/// by using it's URL and tag
pub(crate) fn resolve_folder_name(base: &url::Url, tag: &str) -> String {
    let mut folder_name = base.domain().unwrap().to_owned();
    folder_name.push_str(base.path());
    folder_name.push_str(tag);
    folder_name
}

#[derive(Debug, Clone)]
struct CachedDep {
    entry_path: PathBuf,
    crate_type: CrateType,
    cfg: Config,
    // Whether the dependency came from
    // a remote dependency
    remote: bool,
}

/// Resolves a toml file by either downloading the necessary git repo
/// or it uses the repo on the cache.
/// Downloading will be recursive, so if a package contains packages
/// We need to download those too
pub struct Resolver<'a> {
    cached_packages: HashMap<PathBuf, (CrateId, CachedDep)>,
    driver: &'a mut Driver,
}

impl<'a> Resolver<'a> {
    fn with_driver(driver: &mut Driver) -> Resolver {
        Resolver { cached_packages: HashMap::new(), driver }
    }

    /// Returns the Driver and the backend to use
    /// Note that the backend is ignored in the dependencies.
    /// Since Noir is backend agnostic, this is okay to do.
    /// XXX: Need to handle when a local package changes!
    pub fn resolve_root_config(dir_path: &std::path::Path, np_language: Language) -> Result<Driver, CliError> {
        let mut driver = Driver::new(np_language);
        let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;

        let cfg_path = super::find_package_config(dir_path)?;
        let cfg = super::toml::parse(cfg_path)?;

        let crate_id = driver.create_local_crate(entry_path, crate_type);

        let mut resolver = Resolver::with_driver(&mut driver);
        resolver.resolve_config(crate_id, cfg)?;

        Ok(driver)
    }

    // Resolves a config file by recursively resolving the dependencies in the config
    // Need to solve the case of a project trying to use itself as a dep
    //
    // We do not need to add stdlib, as it's implicitly
    // imported. However, it may be helpful to have the stdlib imported by the
    // package manager.
    fn resolve_config(&mut self, parent_crate: CrateId, cfg: Config) -> Result<(), CliError> {
        // First download and add these top level dependencies crates to the Driver
        for (dep_pkg_name, pkg_src) in cfg.dependencies.iter() {
            let (dir_path, dep_meta) = Resolver::cache_dep(pkg_src)?;

            let (entry_path, crate_type) = (&dep_meta.entry_path, &dep_meta.crate_type);

            if crate_type == &CrateType::Binary {
                return Err(CliError::Generic(format!(
                    "{} is a binary package and so it cannot be depended upon. src : {:?}",
                    dep_pkg_name, pkg_src
                )));
            }

            let crate_id = self.driver.create_non_local_crate(entry_path, *crate_type);
            self.driver.add_dep(parent_crate, crate_id, dep_pkg_name);

            self.cached_packages.insert(dir_path, (crate_id, dep_meta));
        }

        // Resolve all transitive dependencies
        for (dir_path, (crate_id, dep_meta)) in self.cached_packages.iter() {
            if dep_meta.remote && cfg.has_local_path() {
                return Err(CliError::Generic(format!(
                    "remote(git) dependency depends on a local path. \ndependency located at {}",
                    dir_path.display()
                )));
            }
            let mut new_res = Resolver::with_driver(self.driver);
            new_res.resolve_config(*crate_id, dep_meta.cfg.clone())?;
        }
        Ok(())
    }

    /// If the dependency is remote, download the dependency
    /// and return the directory path along with the metadata
    /// Needed to fill the CachedDep struct
    ///
    /// If it's a local path, the same applies, however it will not
    /// be downloaded
    fn cache_dep(dep: &Dependency) -> Result<(PathBuf, CachedDep), CliError> {
        fn retrieve_meta(dir_path: &Path, remote: bool) -> Result<CachedDep, CliError> {
            let (entry_path, crate_type) = super::lib_or_bin(dir_path)?;
            let cfg_path = super::find_package_config(dir_path)?;
            let cfg = super::toml::parse(cfg_path)?;
            Ok(CachedDep { entry_path, crate_type, cfg, remote })
        }

        match dep {
            Dependency::Github { git, tag } => {
                let dir_path = Resolver::resolve_git_dep(git, tag)?;
                let meta = retrieve_meta(&dir_path, true)?;
                Ok((dir_path, meta))
            }
            Dependency::Path { path } => {
                let dir_path = std::path::PathBuf::from(path);
                let meta = retrieve_meta(&dir_path, false)?;
                Ok((dir_path, meta))
            }
        }
    }

    pub fn resolve_git_dep(url: &str, tag: &str) -> Result<PathBuf, CliError> {
        match super::git::clone_git_repo(url, tag) {
            Ok(path) => Ok(path),
            Err(msg) => Err(CliError::Generic(msg)),
        }
    }
}
