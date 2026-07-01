//! Minimal span types used by the ACIR text parser.
//!
//! These mirror a small slice of the public API of `noirc_span` (`Span`, `Position`,
//! `Spanned`). We intentionally do not depend on `noirc_span` because `acir` is published
//! to crates.io as part of the ACVM release set, while `noirc_span` is a compiler crate
//! that is not. Pulling it in would either block the `acir` release or force every
//! transitive compiler crate onto crates.io. Since these types are only used internally
//! by the parser, an inline copy of just what the parser needs is the cheaper option.

pub(super) type Position = u32;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub(crate) struct Span {
    start: u32,
    end: u32,
}

impl Span {
    pub(super) fn inclusive(start: u32, end: u32) -> Span {
        Span { start, end: end + 1 }
    }

    pub(super) fn single_char(position: u32) -> Span {
        Span::inclusive(position, position)
    }

    pub(crate) fn start(&self) -> u32 {
        self.start
    }

    pub(crate) fn end(&self) -> u32 {
        self.end
    }
}

#[derive(Debug, Clone)]
pub(super) struct Spanned<T> {
    pub(super) contents: T,
    span: Span,
}

impl<T> Spanned<T> {
    pub(super) fn from_position(start: Position, end: Position, contents: T) -> Spanned<T> {
        Spanned { span: Span::inclusive(start, end), contents }
    }

    pub(super) fn from(t_span: Span, contents: T) -> Spanned<T> {
        Spanned { span: t_span, contents }
    }

    pub(super) fn span(&self) -> Span {
        self.span
    }
}
