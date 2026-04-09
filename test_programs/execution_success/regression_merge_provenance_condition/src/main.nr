// Regression test: when an if-else merge simplifies to the condition value,
// provenance must not be recorded for it. Otherwise a subsequent merge for
// a different return value can be incorrectly collapsed, producing wrong results.
// See: https://github.com/noir-lang/noir/issues/XXXX
fn main(a: bool) -> pub (bool, bool) {
    if a {
        ((false <= a), a)
    } else {
        (false, true)
    }
}
