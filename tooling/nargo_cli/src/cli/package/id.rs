use std::fmt;
use std::ops::Deref;

use crate::cli::package::name::PackageName;
use crate::cli::package::static_hash_cache::StaticHashCache;
use crate::cli::source::SourceId;
use anyhow::Result;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;
use crate::cli::internal::to_version::ToVersion;

/// See [`PackageIdInner`] for public fields reference.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PackageId(&'static PackageIdInner);

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub struct PackageIdInner {
    pub name: PackageName,
    pub version: Version,
    pub source_id: SourceId,
}

impl PackageId {
    pub fn new(name: PackageName, version: Version, source_id: SourceId) -> Self {
        static CACHE: StaticHashCache<PackageIdInner> = StaticHashCache::new();
        let inner = PackageIdInner {
            name,
            version,
            source_id,
        };
        Self(CACHE.intern(inner))
    }

    pub fn for_test_target(self, target_name: SmolStr) -> Self {
        Self::new(
            PackageName::new(target_name),
            self.version.clone(),
            self.source_id,
        )
    }

    pub fn with_source_id(self, source_id: SourceId) -> Self {
        Self::new(self.name.clone(), self.version.clone(), source_id)
    }

    pub fn is_core(&self) -> bool {
        self.name == PackageName::CORE && self.source_id == SourceId::for_std()
    }

    #[cfg(test)]
    pub(crate) fn from_display_str(string: &str) -> Result<Self> {
        use anyhow::{anyhow, bail, Context};

        let mut s = string.splitn(3, ' ');

        let name =
            PackageName::try_new(s.next().unwrap()).context("invalid displayed PackageId")?;

        let Some(version) = s.next() else {
            bail!("invalid displayed PackageId: missing version");
        };
        let Some(version) = version.strip_prefix('v') else {
            bail!("invalid displayed PackageId: version does not start with letter `v`");
        };
        let version = version
            .to_version()
            .map_err(|err| anyhow!("invalid displayed PackageId: {}", err))?;

        let source_id = match s.next() {
            None => SourceId::default(),
            Some(source_id) => {
                let source_id = if source_id.starts_with('(') && source_id.ends_with(')') {
                    &source_id[1..source_id.len() - 1]
                } else {
                    bail!(
                        "invalid displayed PackageId: source url is not wrapped with parentheses",
                    );
                };
                SourceId::from_display_str(source_id)?
            }
        };

        Ok(PackageId::new(name, version, source_id))
    }

    pub fn to_serialized_string(&self) -> String {
        format!(
            "{} {} ({})",
            self.name,
            self.version,
            self.source_id.to_pretty_url(),
        )
    }

    /// Basename of the tarball that would be created for this package, e.g. `foo-1.2.3`.
    pub fn tarball_basename(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }

    /// Filename of the tarball that would be created for this package, e.g. `foo-1.2.3.tar.zst`.
    pub fn tarball_name(&self) -> String {
        let mut base = self.tarball_basename();
        base.push_str(".tar.zst");
        base
    }
}

impl Deref for PackageId {
    type Target = PackageIdInner;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Serialize for PackageId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&self.to_serialized_string())
    }
}

impl<'de> Deserialize<'de> for PackageId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<PackageId, D::Error> {
        use serde::de::Error;

        let string = String::deserialize(d)?;
        let mut s = string.splitn(3, ' ');

        let name = PackageName::try_new(s.next().unwrap())
            .map_err(|err| Error::custom(format_args!("invalid serialized PackageId: {err}")))?;

        let Some(version) = s.next() else {
            return Err(Error::custom(
                "invalid serialized PackageId: missing version",
            ));
        };
        let version = version
            .to_version()
            .map_err(|err| Error::custom(format_args!("invalid serialized PackageId: {err}")))?;

        let Some(url) = s.next() else {
            return Err(Error::custom(
                "invalid serialized PackageId: missing source url",
            ));
        };
        let url = if url.starts_with('(') && url.ends_with(')') {
            &url[1..url.len() - 1]
        } else {
            return Err(Error::custom(
                "invalid serialized PackageId: source url is not wrapped with parentheses",
            ));
        };
        let source_id = SourceId::from_pretty_url(url).map_err(Error::custom)?;

        Ok(PackageId::new(name, version, source_id))
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)?;

        if !self.source_id.is_default_registry() {
            write!(f, " ({})", self.source_id)?;
        }

        Ok(())
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PackageId({} {} {})",
            self.name, self.version, self.source_id
        )
    }
}