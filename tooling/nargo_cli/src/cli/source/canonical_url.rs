use std::fmt;

use anyhow::{ensure, Result};
use url::Url;
use crate::cli::source::scarb_stable_hash::short_hash;

/// A newtype wrapper around [`Url`] which represents a _canonical_ version of an original URL.
///
/// A _canonical_ url is only intended for internal comparison purposes in Scarb.
/// It's to help paper over mistakes such as depending on `github.com/foo/bar` vs
/// `github.com/foo/bar.git`.
/// This is **only** for internal purposes within Scarb and provides no means to actually read the
/// underlying string value of the [`Url`] it contains.
/// This is intentional, because all fetching should still happen within the context of
/// the original URL.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CanonicalUrl(Url);

impl CanonicalUrl {
    pub fn new(url: &Url) -> Result<Self> {
        ensure!(
            !url.cannot_be_a_base(),
            "invalid url `{url}`: cannot-be-a-base-URLs are not supported"
        );

        let mut url = url.clone();

        // Strip a trailing slash.
        if url.path().ends_with('/') {
            url.path_segments_mut().unwrap().pop_if_empty();
        }

        // For GitHub URLs specifically, just lower-case everything,
        // because GitHub treats both the same.
        if url.host_str() == Some("github.com") {
            url = format!("https{}", &url[url::Position::AfterScheme..])
                .parse()
                .unwrap();
            let path = url.path().to_lowercase();
            url.set_path(&path);
        }

        // Repos can generally be accessed with or without `.git` extension.
        if url.path().ends_with(".git") {
            let last = {
                let last = url.path_segments().unwrap().next_back().unwrap();
                last[..last.len() - 4].to_owned()
            };
            url.path_segments_mut().unwrap().pop().push(&last);
        }

        Ok(Self(url))
    }

    pub fn ident(&self) -> String {
        let base = &self.0;

        let hash = short_hash(base);

        let ident = base
            .path_segments()
            .and_then(|mut s| s.next_back())
            .unwrap_or_default();

        if ident.is_empty() {
            hash
        } else {
            format!("{ident}-{hash}")
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for CanonicalUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for CanonicalUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CanonicalUrl")
            .field(&self.0.as_str())
            .finish()
    }
}