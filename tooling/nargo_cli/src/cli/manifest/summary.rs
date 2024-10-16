use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use dap::types::Checksum;
use semver::Version;
use typed_builder::TypedBuilder;
use crate::cli::manifest::{DepKind, DependencyVersionReq, ManifestDependency};
use crate::cli::package::id::PackageId;
use crate::cli::package::name::PackageName;
#[cfg(doc)]
use crate::core::Manifest;


/// Subset of a [`Manifest`] that contains only the most important information about a package.
/// See [`SummaryInner`] for public fields reference.
/// Construct using [`Summary::builder`].
#[derive(Clone, Debug)]
pub struct Summary(Arc<SummaryInner>);

#[derive(TypedBuilder, Clone, Debug)]
#[builder(builder_type(name = SummaryBuilder))]
#[builder(builder_method(vis = ""))]
#[builder(build_method(into = Summary))]
#[non_exhaustive]
pub struct SummaryInner {
    pub package_id: PackageId,
    #[builder(default)]
    pub dependencies: Vec<ManifestDependency>,
    #[builder(default = false)]
    pub no_core: bool,
    #[builder(default)]
    pub checksum: Option<Checksum>,
}

impl Deref for Summary {
    type Target = SummaryInner;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[doc(hidden)]
impl From<SummaryInner> for Summary {
    fn from(data: SummaryInner) -> Self {
        Self(Arc::new(data))
    }
}

impl Summary {
    pub fn builder() -> SummaryBuilder {
        SummaryInner::builder()
    }

    pub fn set_checksum(&mut self, cksum: Checksum) {
        Arc::make_mut(&mut self.0).checksum = Some(cksum);
    }

    pub fn full_dependencies(&self) -> impl Iterator<Item = &ManifestDependency> {
        self.dependencies.iter().chain(self.implicit_dependencies())
    }

    pub fn filtered_full_dependencies(
        &self,
        dep_filter: DependencyFilter,
    ) -> impl Iterator<Item = &ManifestDependency> {
        self.full_dependencies()
            .filter(move |dep| dep_filter.filter(dep))
    }

    pub fn implicit_dependencies(&self) -> impl Iterator<Item = &ManifestDependency> {
        static CORE_DEPENDENCY: LazyLock<ManifestDependency> = LazyLock::new(|| {
            // NOTE: Pin `core` to exact version, because we know that's the only one we have.
            //todo fix for aztec
            // let cairo_version = crate::version::get().cairo.version.parse().unwrap();
            let cairo_version = Version::new(0, 1, 2);
            ManifestDependency::builder()
                .name(PackageName::CORE)
                .version_req(DependencyVersionReq::exact(&cairo_version))
                .build()
        });
        let mut deps: Vec<&ManifestDependency> = Vec::new();
        if !self.no_core {
            deps.push(&CORE_DEPENDENCY);
        }
        deps.into_iter()
    }

    /// Returns an iterator over dependencies that should be included in registry index record
    /// for this package.
    pub fn publish_dependencies(&self) -> impl Iterator<Item = &ManifestDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.kind == DepKind::Normal)
    }
}

#[derive(Default)]
pub struct DependencyFilter {
    pub do_propagate: bool,
}

impl DependencyFilter {
    pub fn propagation(do_propagate: bool) -> Self {
        Self { do_propagate }
    }

    pub fn filter(&self, dep: &ManifestDependency) -> bool {
        self.do_propagate || dep.kind.is_propagated()
    }
}
