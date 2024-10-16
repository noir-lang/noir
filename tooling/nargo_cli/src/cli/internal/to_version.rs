use anyhow::Result;
use semver::Version;

pub trait ToVersion {
    fn to_version(self) -> Result<Version>;
}

impl ToVersion for Version {
    fn to_version(self) -> Result<Version> {
        Ok(self)
    }
}

impl<'a> ToVersion for &'a str {
    fn to_version(self) -> Result<Version> {
        Version::parse(self.trim())
            .map_err(|_| anyhow::format_err!("cannot parse '{}' as a semver", self))
    }
}

impl<'a> ToVersion for &'a String {
    fn to_version(self) -> Result<Version> {
        (**self).to_version()
    }
}

impl<'a> ToVersion for &'a Version {
    fn to_version(self) -> Result<Version> {
        Ok(self.clone())
    }
}
