// @wrap_comments = true
// @reflow_comments = true
// @format_code_blocks = true
// @comment_width = 35

// Hello world, I just realized
// that this is a long comment
fn one() {}

// First paragraph here that wraps
// nicely.
//
// Second paragraph after a blank
// line break.
fn two() {}

// # Markdown header that is intentionally past the configured width and must not wrap
// body text body text body text
// body text body text
fn three() {}

// - first list item that is long
//   enough to wrap
// - second item
//
// See https://example.com/some/very/long/path/that/exceeds for details.
fn four() {}

/// Build a Foo.
/// @return a fresh Foo with all
/// the goodies inside of it
fn build() {}
