use std::collections::BTreeMap;

use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use indoc::formatdoc;
use nargo::package::Package;
use crate::cli::manifest::{DepKind, DependencyVersionReq, DetailedTomlDependency, Manifest, ManifestDependency, MaybeWorkspace, TargetKind, TomlDependency, TomlManifest, TomlPackage, TomlWorkspaceDependency};
use crate::cli::package::name::PackageName;
use crate::cli::package::source::{DEFAULT_LICENSE_FILE_NAME, DEFAULT_README_FILE_NAME};
// use crate::{
//     core::{
//         DepKind, DependencyVersionReq, DetailedTomlDependency, ManifestDependency, MaybeWorkspace,
//         Package, PackageName, TargetKind, TomlDependency, TomlManifest, TomlPackage,
//         TomlWorkspaceDependency,
//     },
//     DEFAULT_LICENSE_FILE_NAME, DEFAULT_README_FILE_NAME,
// };

pub fn prepare_manifest_for_publish(manifest: Box<Manifest>) -> Result<TomlManifest> {
    let package = Some(generate_package(&manifest));
    let dependencies = Some(generate_dependencies(
        // NOTE: We deliberately do not ask for `full_dependencies` here, because
        // we do not want to emit requirements for built-in packages like `core`.
        &manifest.summary.dependencies,
        DepKind::Normal,
    )?);

    let tool = manifest.metadata.tool_metadata.clone().map(|m| {
        m.into_iter()
            .map(|(k, v)| (k, MaybeWorkspace::Defined(v)))
            .collect()
    });

    //todo fix for aztec
    // let cairo_plugin = match pkg.target(&TargetKind::CAIRO_PLUGIN) {
    //     None => None,
    //     Some(_) => todo!("Packaging Cairo plugins is not implemented yet."),
    // };

    Ok(TomlManifest {
        package,
        workspace: None,
        dependencies,
        dev_dependencies: None,
        lib: None,
        //todo fix for aztec
        // cairo_plugin,
        cairo_plugin: None,
        test: None,
        target: None,
        cairo: None,
        profile: None,
        scripts: None,
        tool,
        features: None,
    })
}

fn generate_package(manifest: &Box<Manifest>) -> Box<TomlPackage> {
    let summary = &manifest.summary;
    let metadata = &manifest.metadata;
    Box::new(TomlPackage {
        name: summary.package_id.name.clone(),
        version: MaybeWorkspace::Defined(summary.package_id.version.clone()),
        // edition: Some(MaybeWorkspace::Defined(pkg.manifest.edition)),
        publish: (!manifest.publish).then_some(false),
        authors: metadata.authors.clone().map(MaybeWorkspace::Defined),
        urls: metadata.urls.clone(),
        description: metadata.description.clone().map(MaybeWorkspace::Defined),
        documentation: metadata.documentation.clone().map(MaybeWorkspace::Defined),
        homepage: metadata.homepage.clone().map(MaybeWorkspace::Defined),
        keywords: metadata.keywords.clone().map(MaybeWorkspace::Defined),
        license: metadata.license.clone().map(MaybeWorkspace::Defined),
        license_file: metadata
            .license_file
            .clone()
            .map(|_| MaybeWorkspace::Defined(Utf8PathBuf::from(DEFAULT_LICENSE_FILE_NAME))),
        readme: metadata
            .readme
            .clone()
            .map(|_| MaybeWorkspace::Defined((Utf8PathBuf::from(DEFAULT_README_FILE_NAME)).into())),
        repository: metadata.repository.clone().map(MaybeWorkspace::Defined),
        no_core: summary.no_core.then_some(true),
        cairo_version: metadata.cairo_version.clone().map(MaybeWorkspace::Defined),
        experimental_features: manifest.experimental_features.clone(),
    })
}

fn generate_dependencies(
    deps: &[ManifestDependency],
    kind: DepKind,
) -> Result<BTreeMap<PackageName, MaybeWorkspace<TomlDependency, TomlWorkspaceDependency>>> {
    deps.iter()
        .filter(|dep| dep.kind == kind)
        .map(|dep| {
            let name = dep.name.clone();
            let toml_dep = generate_dependency(dep)?;
            Ok((name, MaybeWorkspace::Defined(toml_dep)))
        })
        .collect()
}

fn generate_dependency(dep: &ManifestDependency) -> Result<TomlDependency> {
    assert!(
        !dep.source_id.is_std(),
        "Built-in dependencies should not be included in published manifests."
    );

    let version = Some(match &dep.version_req {
        DependencyVersionReq::Req(req) => req.clone(),

        // Ignore what is in the lock file.
        DependencyVersionReq::Locked { req, .. } => req.clone(),

        // This case is triggered by dependencies like this:
        //
        // [dependencies]
        // foo = { path = "../foo" }
        DependencyVersionReq::Any => {
            bail!(formatdoc! {
                r#"
                    dependency `{name}` does not specify a version requirement
                    note: all dependencies must have a version specified when packaging
                    note: the `{kind}` specification will be removed from dependency declaration 
                "#,
                name = dep.name,
                kind = dep.source_id.kind.primary_field(),
            })
        }
    });

    Ok(TomlDependency::Detailed(Box::new(DetailedTomlDependency {
        version,

        // Erase path information, effectively making the dependency default registry-based.
        path: None,

        // Same for Git specification.
        git: None,
        branch: None,
        tag: None,
        rev: None,

        // Unless it is default registry, expand registry specification to registry URL.
        //
        // NOTE: Default registry will reject packages with dependencies from other registries.
        registry: if dep.source_id.is_registry() && !dep.source_id.is_default_registry() {
            Some(dep.source_id.url.clone())
        } else {
            None
        },
    })))
}
