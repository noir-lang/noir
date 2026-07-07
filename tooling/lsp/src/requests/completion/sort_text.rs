/// Local variables should be suggested before anything else.
pub(super) fn local_variable_sort_text() -> String {
    "a".to_string()
}

/// Sort text for "new" methods: we want these to show up before anything else,
/// if we are completing at something like `Foo::`
pub(super) fn new_sort_text() -> String {
    "b".to_string()
}

/// This is the default sort text.
pub(super) fn default_sort_text() -> String {
    "c".to_string()
}

/// We want crates and modules to show up after other things (for example
/// local variables, functions or types)
pub(super) fn crate_or_module_sort_text() -> String {
    "d".to_string()
}

/// Sort text for auto-import items. We want these to show up after local definitions.
pub(super) fn auto_import_sort_text() -> String {
    "e".to_string()
}

/// When completing something like `Foo::`, we want to show methods that take
/// self after the other ones.
pub(super) fn self_mismatch_sort_text() -> String {
    "f".to_string()
}

/// We want to show operator methods last.
pub(super) fn operator_sort_text() -> String {
    "g".to_string()
}

/// If a name begins with underscore it's likely something that's meant to
/// be private (but visibility doesn't exist everywhere yet, so for now
/// we assume that)
pub(super) fn underscore_sort_text() -> String {
    "h".to_string()
}
