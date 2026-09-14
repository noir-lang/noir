use std::fmt;
use std::str::FromStr;

use thiserror::Error;
use url::Url;

/// A URL pointing at an oracle resolver RPC endpoint, parsed from a CLI argument.
///
/// The string must parse as a URL and have an `http` or `https` scheme.
#[derive(Debug, Clone)]
pub struct OracleResolverUrl(Url);

impl OracleResolverUrl {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for OracleResolverUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum OracleResolverUrlParseError {
    #[error("`{input}` is not a valid URL: {source}")]
    InvalidUrl { input: String, source: url::ParseError },
    #[error("URL scheme must be `http` or `https`, got `{0}`")]
    UnsupportedScheme(String),
}

impl FromStr for OracleResolverUrl {
    type Err = OracleResolverUrlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s).map_err(|source| OracleResolverUrlParseError::InvalidUrl {
            input: s.to_string(),
            source,
        })?;
        match url.scheme() {
            "http" | "https" => Ok(OracleResolverUrl(url)),
            other => Err(OracleResolverUrlParseError::UnsupportedScheme(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OracleResolverUrl, OracleResolverUrlParseError};

    #[test]
    fn parses_valid_http_url() {
        let url: OracleResolverUrl = "http://localhost:8080".parse().unwrap();
        assert_eq!(url.as_str(), "http://localhost:8080/");
    }

    #[test]
    fn parses_valid_https_url() {
        let url: OracleResolverUrl = "https://example.com:1234/oracle".parse().unwrap();
        assert_eq!(url.as_str(), "https://example.com:1234/oracle");
    }

    #[test]
    fn rejects_url_without_scheme() {
        let err = "localhost:8080".parse::<OracleResolverUrl>().unwrap_err();
        assert!(matches!(err, OracleResolverUrlParseError::UnsupportedScheme(_)));
    }

    #[test]
    fn rejects_garbage_input() {
        let err = "not a url".parse::<OracleResolverUrl>().unwrap_err();
        assert!(matches!(err, OracleResolverUrlParseError::InvalidUrl { .. }));
    }

    #[test]
    fn rejects_relative_url() {
        let err = "/just/a/path".parse::<OracleResolverUrl>().unwrap_err();
        assert!(matches!(err, OracleResolverUrlParseError::InvalidUrl { .. }));
    }

    #[test]
    fn rejects_non_http_scheme() {
        let err = "ftp://example.com".parse::<OracleResolverUrl>().unwrap_err();
        match err {
            OracleResolverUrlParseError::UnsupportedScheme(scheme) => assert_eq!(scheme, "ftp"),
            OracleResolverUrlParseError::InvalidUrl { .. } => {
                panic!("unexpected error: {err:?}")
            }
        }
    }

    #[test]
    fn rejects_file_scheme() {
        let err = "file:///etc/passwd".parse::<OracleResolverUrl>().unwrap_err();
        assert!(matches!(err, OracleResolverUrlParseError::UnsupportedScheme(_)));
    }
}
