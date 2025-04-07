use convert_case::{Case, Casing};

/// Returns true if name matches a prefix written in code.
/// `prefix` must already be in snake case.
/// This method splits both name and prefix by underscore,
/// then checks that every part of name starts with a part of
/// prefix, in order.
///
/// For example:
///
/// // "merk" and "ro" match "merkle" and "root" and are in order
/// name_matches("compute_merkle_root", "merk_ro") == true
///
/// // "ro" matches "root", but "merkle" comes before it, so no match
/// name_matches("compute_merkle_root", "ro_mer") == false
///
/// // neither "compute" nor "merkle" nor "root" start with "oot"
/// name_matches("compute_merkle_root", "oot") == false
pub(crate) fn name_matches(name: &str, prefix: &str) -> bool {
    let name = name.to_case(Case::Snake);
    snake_name_matches(&name, prefix)
}

/// Like `name_matches` but assumes `name` is already in snake-case.
pub(crate) fn snake_name_matches(name: &str, prefix: &str) -> bool {
    let name_parts: Vec<&str> = name.split('_').collect();

    let mut last_index: i32 = -1;
    for prefix_part in prefix.split('_') {
        // Look past parts we already matched
        let offset = if last_index >= 0 { last_index as usize + 1 } else { 0 };

        if let Some(mut name_part_index) =
            name_parts.iter().skip(offset).position(|name_part| name_part.starts_with(prefix_part))
        {
            // Need to adjust the index if we skipped some segments
            name_part_index += offset;

            if last_index >= name_part_index as i32 {
                return false;
            }
            last_index = name_part_index as i32;
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod completion_name_matches_tests {
    use crate::name_match::name_matches;

    #[test]
    fn test_name_matches() {
        assert!(name_matches("foo", "foo"));
        assert!(name_matches("foo_bar", "bar"));
        assert!(name_matches("FooBar", "foo"));
        assert!(name_matches("FooBar", "bar"));
        assert!(name_matches("FooBar", "foo_bar"));
        assert!(name_matches("bar_baz", "bar_b"));

        assert!(!name_matches("foo_bar", "o_b"));
    }
}
