use std::fmt;

use semver::{Comparator, Op, Version, VersionReq};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum DependencyVersionReq {
    #[default]
    Any,
    Req(VersionReq),
    Locked {
        exact: Version,
        req: VersionReq,
    },
}

impl DependencyVersionReq {
    pub fn exact(version: &Version) -> Self {
        Self::Req(VersionReq {
            comparators: vec![Comparator {
                op: Op::Exact,
                major: version.major,
                minor: Some(version.minor),
                patch: Some(version.patch),
                pre: version.pre.clone(),
            }],
        })
    }

    /// Evaluate whether the given `Version` satisfies the version requirement
    /// described by `self`.
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            DependencyVersionReq::Any => true,
            DependencyVersionReq::Req(req) => req.matches(version),
            DependencyVersionReq::Locked { exact, .. } => {
                exact.major == version.major
                    && exact.minor == version.minor
                    && exact.patch == version.patch
                    && exact.pre == version.pre
            }
        }
    }
}

impl From<VersionReq> for DependencyVersionReq {
    fn from(req: VersionReq) -> Self {
        DependencyVersionReq::Req(req)
    }
}

impl From<DependencyVersionReq> for VersionReq {
    fn from(req: DependencyVersionReq) -> Self {
        match req {
            DependencyVersionReq::Any => VersionReq::STAR,
            DependencyVersionReq::Req(req) => req,
            DependencyVersionReq::Locked { req, .. } => req,
        }
    }
}

impl fmt::Display for DependencyVersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyVersionReq::Any => f.write_str("*"),
            DependencyVersionReq::Req(req) => fmt::Display::fmt(req, f),
            DependencyVersionReq::Locked { req, .. } => fmt::Display::fmt(req, f),
        }
    }
}
