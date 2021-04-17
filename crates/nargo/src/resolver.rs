use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use acvm::BackendPointer;
use ark_bn254::Fr;
use noir_field::Bn254Scalar;
use noirc_driver::Driver;
use noirc_frontend::graph::{CrateId, CrateType};

use crate::{
    toml::{Config, Dependency},
    write_stderr,
};

#[allow(dead_code)]
enum PossibleDrivers {
    Bn254(Driver<Bn254Scalar>),
}

/// Creates a unique folder name for a github repo
/// by using it's url and tag
pub(crate) fn resolve_folder_name(base: &url::Url, tag: &str) -> String {
    let mut folder_name = base.domain().unwrap().to_owned();
    folder_name.push_str(base.path());
    folder_name.push_str(&tag);
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
// TODO: this Resolver should probably not return a polymorphic result until
// and unless we make it meaningfully inspect a (necessarily dynamic) configuration to return a
// Driver  using one backend or another.
pub struct Resolver<'a> {
    cached_packages: HashMap<PathBuf, (CrateId, CachedDep)>,
    driver: &'a mut Driver<Fr>,
}

impl<'a> Resolver<'a> {
    fn with_driver(driver: &mut Driver<Fr>) -> Resolver {
        Resolver {
            cached_packages: HashMap::new(),
            driver,
        }
    }

    fn default_backend() -> BackendPointer {
        acvm::fetch_by_name("csat_3_plonk_aztec").unwrap()
    }

    fn resolve_backend(name: Option<&str>) -> BackendPointer {
        match name {
            None => Resolver::default_backend(),
            Some(name) => {
                let backend = acvm::fetch_by_name(&name);
                match backend {
                    None => write_stderr(&format!("unknown backend: {}", name)),
                    Some(backend) => backend,
                }
            }
        }
    }

    /// Returns the Driver and the Backend to use
    /// Note that the backend is ignored in the dependencies.
    /// Since Noir is backend agnostic, this is okay to do.
    /// XXX: Need to handle when a local package changes!
    pub fn resolve_root_config(dir_path: &std::path::Path) -> (Driver<Fr>, BackendPointer) {
        // XXX: We figure out the field in this function via the toml file
        let mut driver = Driver::<Bn254Scalar>::new();

        let (entry_path, crate_type) = super::lib_or_bin(&dir_path);

        let cfg_path = super::find_package_config(&dir_path);
        let cfg = super::toml::parse(cfg_path);

        let backend = Resolver::resolve_backend(cfg.package.backend.as_deref());

        let crate_id = driver.create_local_crate(entry_path, crate_type);

        let mut resolver = Resolver::with_driver(&mut driver);
        resolver.resolve_config(crate_id, cfg);

        (driver, backend)
    }

    // Resolves a config file by recursively resolving the dependencies in the config
    // Need to solve the case of a project trying to use itself as a dep
    //
    // We do not need to add stdlib, as it's implicitly
    // imported. However, it may be helpful to have the stdlib imported by the
    // package manager.
    fn resolve_config(&mut self, parent_crate: CrateId, cfg: Config) {
        // First download and add these top level dependencies crates to the Driver
        for (dep_pkg_name, pkg_src) in cfg.dependencies.iter() {
            let (dir_path, dep_meta) = Resolver::cache_dep2(pkg_src);

            let (entry_path, crate_type) = (&dep_meta.entry_path, &dep_meta.crate_type);

            if crate_type == &CrateType::Binary {
                super::write_stderr(&format!(
                    "{} is a binary package and so it cannot be depended upon. src : {:?}",
                    dep_pkg_name, pkg_src
                ));
            }

            let crate_id = self.driver.create_non_local_crate(&entry_path, *crate_type);
            self.driver.add_dep(parent_crate, crate_id, dep_pkg_name);

            self.cached_packages.insert(dir_path, (crate_id, dep_meta));
        }

        // Resolve all transitive dependencies
        for (dir_path, (crate_id, dep_meta)) in self.cached_packages.iter() {
            if dep_meta.remote && cfg.has_local_path() {
                super::write_stderr(&format!(
                    "remote(git) dependency depends on a local path. \ndependency located at {}",
                    dir_path.display()
                ))
            }
            let mut new_res = Resolver::with_driver(self.driver);
            new_res.resolve_config(*crate_id, dep_meta.cfg.clone());
        }
    }

    /// If the dependency is remote, download the dependency
    /// and return the directory path along with the metadata
    /// Needed to fill the CachedDep struct
    ///
    /// If it's a local path, the same applies, however it will not
    /// be downloaded
    fn cache_dep2(dep: &Dependency) -> (PathBuf, CachedDep) {
        fn retrieve_meta(dir_path: &Path, remote: bool) -> CachedDep {
            let (entry_path, crate_type) = super::lib_or_bin(&dir_path);
            let cfg_path = super::find_package_config(&dir_path);
            let cfg = super::toml::parse(cfg_path);
            CachedDep {
                entry_path,
                crate_type,
                cfg,
                remote,
            }
        }

        match dep {
            Dependency::Github { git, tag } => {
                let dir_path = Resolver::resolve_git_dep(git, tag);
                let meta = retrieve_meta(&dir_path, true);
                (dir_path, meta)
            }
            Dependency::Path { path: _ } => todo!(),
        }
    }

    pub fn resolve_git_dep(url: &str, tag: &str) -> PathBuf {
        match super::git::clone_git_repo(url, tag) {
            Ok(path) => path,
            Err(msg) => crate::write_stderr(&msg),
        }
    }
}
