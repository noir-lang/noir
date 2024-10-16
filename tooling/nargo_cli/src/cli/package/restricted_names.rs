//! Helpers for validating and checking names.

use std::path::Path;

/// Checks if name is a Cairo keyword
pub fn is_keyword(name: &str) -> bool {
    [
        "as",
        "assert",
        "break",
        "const",
        "continue",
        "do",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "hint",
        "if",
        "impl",
        "implicits",
        "in",
        "let",
        "loop",
        "macro",
        "match",
        "mod",
        "move",
        "mut",
        "nopanic",
        "of",
        "pub",
        "ref",
        "return",
        "self",
        "static",
        "static_assert",
        "struct",
        "super",
        "trait",
        "true",
        "try",
        "type",
        "typeof",
        "unsafe",
        "use",
        "where",
        "while",
        "with",
        "yield",
    ]
    .contains(&name)
}

/// Checks if name is restricted on Windows platforms.
pub fn is_windows_restricted(name: &str) -> bool {
    [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ]
    .contains(&name)
}

/// Checks the entire path for names restricted on Windows platforms.
pub fn is_windows_restricted_path(path: &Path) -> bool {
    path.iter()
        .filter_map(|c| c.to_str())
        .filter_map(|s| s.split('.').next())
        .any(is_windows_restricted)
}

/// Checks if name equals `core` or `starknet`
pub fn is_internal(name: &str) -> bool {
    [
        //todo fix these names to be correct for Aztec
        "core",
        "starknet",
    ]
    .contains(&name)
}
