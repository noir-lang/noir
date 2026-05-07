//! Local registry of types participating in tagged-map serialization.
//!
//! Built once per encode/decode call by walking the type graph from a top-level
//! type via [`MsgpackTagged::register_into`]. The wrapper Serializer/Deserializer
//! consults this registry to translate between serde field names and integer tags.

use std::any::TypeId;
use std::collections::HashMap;

use crate::{MsgpackTagged, Tag};

/// A registered type's metadata.
#[derive(Debug)]
pub struct Entry {
    /// Used to detect serde-name collisions between two different Rust types.
    type_id: TypeId,
    tags: &'static [(Tag, &'static str)],
    reserved: &'static [Tag],
    defaults: &'static [Tag],
    allow_unknown_tags: bool,
}

impl Entry {
    pub fn tags(&self) -> &'static [(Tag, &'static str)] {
        self.tags
    }

    pub fn reserved(&self) -> &'static [Tag] {
        self.reserved
    }

    pub fn defaults(&self) -> &'static [Tag] {
        self.defaults
    }

    pub fn allow_unknown_tags(&self) -> bool {
        self.allow_unknown_tags
    }

    /// Look up a field's tag by its serde name. O(N) over the static TAGS
    /// array — acceptable for the small (typically 3-30) field counts of ACIR
    /// types; if a profile ever shows this hot, the registry can precompute
    /// HashMap views.
    pub fn tag_for(&self, field_name: &str) -> Option<Tag> {
        self.tags.iter().find(|(_, name)| *name == field_name).map(|(t, _)| *t)
    }

    /// Look up a field's serde name by its tag.
    pub fn field_for(&self, tag: Tag) -> Option<&'static str> {
        self.tags.iter().find(|(t, _)| *t == tag).map(|(_, name)| *name)
    }

    /// Whether `tag` is in the type's reserved list (a retired tag from an
    /// older schema version — silently skipped on decode).
    pub fn is_reserved(&self, tag: Tag) -> bool {
        self.reserved.contains(&tag)
    }

    /// Whether the field at `tag` is marked `#[tag(N, default)]` — i.e.,
    /// wire-tolerant: the decoder should fill in `T::default()` if the tag
    /// is missing rather than raising an error.
    pub fn is_default(&self, tag: Tag) -> bool {
        self.defaults.contains(&tag)
    }
}

/// A registry of types participating in tagged-map serialization.
#[derive(Default, Debug)]
pub struct TagRegistry {
    entries: HashMap<&'static str, Entry>,
}

impl TagRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a type under its serde name.
    ///
    /// Returns `true` if this type was newly inserted — the caller (typically a
    /// macro-generated `register_into` body) should then recurse into the type's
    /// field types. Returns `false` if the same type was already registered,
    /// short-circuiting the recursive walk.
    ///
    /// **Panics** if a *different* Rust type is already registered under the same
    /// `name` — that signals a real serde-name collision, which the user must
    /// resolve with `#[serde(rename = "...")]` on one of the types.
    pub fn try_insert<T: MsgpackTagged>(&mut self, name: &'static str) -> bool {
        use std::collections::hash_map::Entry as HashEntry;
        match self.entries.entry(name) {
            HashEntry::Vacant(slot) => {
                slot.insert(Entry {
                    type_id: TypeId::of::<T>(),
                    tags: T::TAGS,
                    reserved: T::RESERVED,
                    defaults: T::DEFAULTS,
                    allow_unknown_tags: T::ALLOW_UNKNOWN_TAGS,
                });
                true
            }
            HashEntry::Occupied(slot) => {
                if slot.get().type_id == TypeId::of::<T>() {
                    false
                } else {
                    panic!(
                        "MsgpackTagged registry collision: serde name {name:?} is registered for two different Rust types — disambiguate with #[serde(rename = \"...\")] on one of them"
                    );
                }
            }
        }
    }

    /// Look up a type's entry by serde name. Returns `None` if the type was
    /// never registered — the wrapper decides whether that's an error
    /// (encode-side) or a clean failure to decode (decode-side).
    pub fn get(&self, name: &str) -> Option<&Entry> {
        self.entries.get(name)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo;
    impl MsgpackTagged for Foo {
        const TAGS: &'static [(Tag, &'static str)] = &[(0, "a"), (1, "b")];
        const RESERVED: &'static [Tag] = &[3];
        const DEFAULTS: &'static [Tag] = &[1];
        const ALLOW_UNKNOWN_TAGS: bool = true;
        fn register_into(_reg: &mut TagRegistry) {}
    }

    struct Bar;
    impl MsgpackTagged for Bar {
        const TAGS: &'static [(Tag, &'static str)] = &[(0, "x")];
        fn register_into(_reg: &mut TagRegistry) {}
    }

    #[test]
    fn try_insert_returns_true_on_first_insert() {
        let mut reg = TagRegistry::new();
        assert!(reg.try_insert::<Foo>("Foo"));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn try_insert_returns_false_on_idempotent_reinsert() {
        let mut reg = TagRegistry::new();
        assert!(reg.try_insert::<Foo>("Foo"));
        assert!(!reg.try_insert::<Foo>("Foo"));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    #[should_panic(expected = "registry collision")]
    fn try_insert_panics_on_name_collision_between_different_types() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Foo>("Same");
        reg.try_insert::<Bar>("Same");
    }

    #[test]
    fn distinct_names_for_different_types_coexist() {
        let mut reg = TagRegistry::new();
        assert!(reg.try_insert::<Foo>("Foo"));
        assert!(reg.try_insert::<Bar>("Bar"));
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn get_returns_entry_for_registered_type() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Foo>("Foo");
        let entry = reg.get("Foo").unwrap();
        assert_eq!(entry.tags(), Foo::TAGS);
        assert_eq!(entry.reserved(), Foo::RESERVED);
        assert!(entry.allow_unknown_tags());
    }

    #[test]
    fn get_returns_none_for_unknown_name() {
        let reg = TagRegistry::new();
        assert!(reg.get("Anything").is_none());
    }

    #[test]
    fn entry_tag_for_finds_known_fields() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Foo>("Foo");
        let entry = reg.get("Foo").unwrap();
        assert_eq!(entry.tag_for("a"), Some(0));
        assert_eq!(entry.tag_for("b"), Some(1));
        assert_eq!(entry.tag_for("missing"), None);
    }

    #[test]
    fn entry_field_for_finds_known_tags() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Foo>("Foo");
        let entry = reg.get("Foo").unwrap();
        assert_eq!(entry.field_for(0), Some("a"));
        assert_eq!(entry.field_for(1), Some("b"));
        assert_eq!(entry.field_for(99), None);
    }

    #[test]
    fn entry_is_reserved_only_for_listed_tags() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Foo>("Foo");
        let entry = reg.get("Foo").unwrap();
        assert!(entry.is_reserved(3));
        assert!(!entry.is_reserved(0));
        assert!(!entry.is_reserved(99));
    }

    #[test]
    fn entry_is_default_only_for_listed_tags() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Foo>("Foo");
        let entry = reg.get("Foo").unwrap();
        assert!(entry.is_default(1), "Foo's tag 1 (`b`) is in DEFAULTS");
        assert!(!entry.is_default(0), "tag 0 (`a`) is not defaulted");
        assert!(!entry.is_default(99), "unknown tags are not defaulted");
    }

    #[test]
    fn empty_registry() {
        let reg = TagRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }
}
