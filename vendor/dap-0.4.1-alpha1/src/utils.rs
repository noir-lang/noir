use std::fmt::{Display, Formatter};

/// A struct representing a version of the DAP specification.
/// This version corresponds to the [changelog](https://microsoft.github.io/debug-adapter-protocol/changelog)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
  /// Major version.
  pub major: i64,
  /// Minor version.
  pub minor: i64,
  /// Patch version. Historically, this is "x" in the changelog for most versions. That value
  /// is represented as `None` here.
  pub patch: Option<i64>,
  /// The git commit in the DAP repo that corresponds to this version.
  /// Please note that historically, versions (as of 1.62.x) are not tagged in the DAP repo.
  /// Until that changes, we are using the commit that updates the JSON-schema in the gh-pages
  /// branch.
  pub git_commit: String,
}

impl Display for Version {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self.patch {
      Some(patch) => write!(f, "{}.{}.{}", self.major, self.minor, patch),
      None => write!(f, "{}.{}.x", self.major, self.minor),
    }
  }
}

/// Returns the version of the DAP specification that this crate implements.
///
/// Please note that historically, the DAP changelog hasn't been super accurate and the
/// versions (as of 1.62.x) are not tagged in the DAP repo. Until that changes, we are
/// using the commit that *adds the corresponding JSON-schema in the **gh-pages** branch*.
pub fn get_spec_version() -> Version {
  Version {
    major: 1,
    minor: 62,
    patch: None,
    git_commit: "7f284b169ecd19602487eb4d290ae651d4398ce7".to_string(),
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_version_display() {
    let version = Version {
      major: 1,
      minor: 62,
      patch: None,
      git_commit: "7f284b169ecd19602487eb4d290ae651d4398ce7".to_string(),
    };
    assert_eq!(version.to_string(), "1.62.x");

    let version = Version {
      major: 1,
      minor: 62,
      patch: Some(1),
      git_commit: "7f284b169ecd19602487eb4d290ae651d4398ce7".to_string(),
    };
    assert_eq!(version.to_string(), "1.62.1");
  }
}
