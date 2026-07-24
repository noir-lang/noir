//! The registry of lints understood by the compiler.
//!
//! A "lint" is an opinionated warning: a diagnostic that flags legitimate but
//! usually-undesirable code (an unused item, an unnecessary `mut`, …). Each lint
//! has a stable, human-readable slug (`dead_code`, `unused_variables`, …) that a
//! user can name in an `#[allow(...)]` attribute to silence it.
//!
//! [`Lint`] is the closed set of valid slugs. Making the set closed is what lets the
//! parser reject an unrecognised slug: parsing `#[allow(<slug>)]`
//! (`parser/parser/attributes.rs`) looks the slug up with [`Lint::from_slug`], and an
//! unknown slug raises the `UnknownLint` warning rather than being accepted as an inert
//! no-op the author mistakes for a working suppression. That warning is a parser warning,
//! which never blocks elaboration, so the lint the author *meant* to silence still fires.
//!
//! Consumers that honour `#[allow(...)]` compare against a [`Lint`] (via
//! [`crate::token::Attributes::has_allow`] / `SecondaryAttributeKind::is_allow`) rather
//! than a bare string, so the slug spellings live only in [`Lint::slug`].
//!
//! [`Lint`] lists only the lints `#[allow(...)]` actually silences: a slug is "known" only
//! when naming it has a real effect. See `design/lints.md` for the rationale and the
//! intended direction.

use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// A lint the compiler knows about and that can be named in `#[allow(...)]`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum Lint {
    /// Item is never used (`dead_code`).
    DeadCode,
    /// Local variable is never used (`unused_variables`).
    UnusedVariables,
    /// Binding is marked `mut` but never mutated (`unused_mut`).
    UnusedMut,
}

impl Lint {
    /// The stable, human-readable slug used to refer to this lint in source
    /// (e.g. `#[allow(dead_code)]`).
    pub fn slug(self) -> &'static str {
        match self {
            Lint::DeadCode => "dead_code",
            Lint::UnusedVariables => "unused_variables",
            Lint::UnusedMut => "unused_mut",
        }
    }

    /// Look up a lint by its slug, returning `None` if the slug is not recognised.
    pub fn from_slug(slug: &str) -> Option<Lint> {
        Lint::iter().find(|lint| lint.slug() == slug)
    }
}

impl fmt::Display for Lint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

#[cfg(test)]
mod tests {
    use super::Lint;
    use strum::IntoEnumIterator;

    #[test]
    fn slug_round_trips() {
        for lint in Lint::iter() {
            assert_eq!(Lint::from_slug(lint.slug()), Some(lint));
        }
    }

    #[test]
    fn unknown_slug_is_none() {
        assert_eq!(Lint::from_slug("dead_cod"), None);
        assert_eq!(Lint::from_slug(""), None);
    }
}
