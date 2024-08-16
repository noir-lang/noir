/// Sort text for "new" methods: we want these to show up before anything else,
/// if we are completing at something like `Foo::`
pub(super) fn new_sort_text() -> String {
    "3".to_string()
}

/// This is the default sort text.
pub(super) fn default_sort_text() -> String {
    "5".to_string()
}

/// Sort text for auto-import items. We want these to show up after local definitions.
pub(super) fn auto_import_sort_text() -> String {
    "6".to_string()
}

/// When completing something like `Foo::`, we want to show methods that take
/// self after the other ones.
pub(super) fn self_mismatch_sort_text() -> String {
    "7".to_string()
}

/// We want to show operator methods last.
pub(super) fn operator_sort_text() -> String {
    "8".to_string()
}

/// If a name begins with underscore it's likely something that's meant to
/// be private (but visibility doesn't exist everywhere yet, so for now
/// we assume that)
pub(super) fn underscore_sort_text() -> String {
    "9".to_string()
}
