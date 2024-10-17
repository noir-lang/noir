use std::fmt;
use std::str::FromStr;

use semver::VersionReq;
use crate::cli::package::name::PackageName;

/// Reference to a package to be added as a dependency.
///
/// See `scarb add` help for more info.
#[derive(Clone, Debug, Default)]
pub struct DepId {
    pub name: Option<PackageName>,
    pub version_req: Option<VersionReq>,
}

impl DepId {
    pub fn unspecified() -> Self {
        Self::default()
    }
}

impl FromStr for DepId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut dep = DepId::default();

        if s.is_empty() {
            return Ok(dep);
        }

        let mut s = s.split('@');
        let Some(name) = s.next() else {
            return Ok(dep);
        };
        dep.name = Some(name.parse()?);

        let Some(version_req) = s.next() else {
            return Ok(dep);
        };
        dep.version_req = Some(version_req.parse()?);

        Ok(dep)
    }
}

impl fmt::Display for DepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{name}")?;
        }

        if let Some(version_req) = &self.version_req {
            write!(f, "@{version_req}")?;
        }

        Ok(())
    }
}