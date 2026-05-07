//! Local registry of types participating in tagged-map serialization.
//!
//! Built once per encode/decode call by walking the type graph from a top-level
//! type via [`MsgpackTagged::register_into`]. The wrapper Serializer/Deserializer
//! consults this registry to translate between serde field names and integer tags.
//!
//! ## Wire-shape model
//!
//! A tagged type is one of two algebraic shapes:
//!
//! * A [`Product`] — a fixed list of `(tag, name)` field entries. Used for
//!   structs, tuple structs, and (recursively) for an enum variant's payload.
//! * A [`Sum`] — a discriminated union of [`Variant`]s, each carrying its own
//!   `Product` payload.
//!
//! Both shapes are unified under [`Tagged`], which is the only thing the trait
//! exposes (via the `TAGGED` associated const). The registry stores `Tagged`
//! values keyed by serde name and routes wrapper code through the matching arm.
//!
//! Every type used here ([`Tagged`], [`Product`], [`Variant`], [`Sum`]) is
//! `Copy` with public fields — they're built directly in `const` context by
//! the derive macro and read flatly from the trait, so there's no
//! encapsulated state to protect.

use std::any::TypeId;
use std::collections::HashMap;

use crate::{MsgpackTagged, Tag};

/// The wire shape of a tagged type, used both at the top level (in
/// `MsgpackTagged::TAGGED`) and recursively inside variant payloads.
#[derive(Clone, Copy, Debug)]
pub enum Tagged {
    /// A struct, tuple struct, or any other product-shaped wire type.
    Product(Product),
    /// An enum: a discriminated union of variants.
    Sum(Sum),
}

impl Tagged {
    /// Borrow the inner [`Product`] if this is a product shape.
    pub fn as_product(self) -> Option<Product> {
        match self {
            Tagged::Product(p) => Some(p),
            Tagged::Sum(_) => None,
        }
    }

    /// Borrow the inner [`Sum`] if this is a sum shape.
    pub fn as_sum(self) -> Option<Sum> {
        match self {
            Tagged::Sum(s) => Some(s),
            Tagged::Product(_) => None,
        }
    }
}

/// A product type — a fixed list of named, integer-tagged fields. Used for
/// top-level structs/tuple structs *and* for an enum variant's payload (a
/// variant is structurally just a struct hung off a tag).
///
/// `fields` is in tag-ascending order (the canonical wire order). `reserved`
/// lists tags previously used by this product and now retired — purely
/// compile-time metadata that prevents reuse, never affects decode behavior.
/// `defaults` lists the subset of tags whose decoder is allowed to fall back
/// to `T::default()` when missing on the wire (i.e., `#[tag(N, default)]`).
/// `allow_unknown_tags` opts the decoder into silently skipping fields whose
/// tag isn't in `fields` or `reserved`.
#[derive(Clone, Copy, Debug)]
pub struct Product {
    pub fields: &'static [(Tag, &'static str)],
    pub reserved: &'static [Tag],
    pub defaults: &'static [Tag],
    pub allow_unknown_tags: bool,
}

impl Product {
    /// Look up a field's tag by its serde name. O(N) over `fields` —
    /// acceptable for the small (typically 3-30) field counts of ACIR types;
    /// if a profile ever shows this hot, the registry can precompute
    /// HashMap views.
    pub fn tag_for(self, field_name: &str) -> Option<Tag> {
        self.fields.iter().find(|(_, name)| *name == field_name).map(|(t, _)| *t)
    }

    /// Look up a field's serde name by its tag.
    pub fn field_for(self, tag: Tag) -> Option<&'static str> {
        self.fields.iter().find(|(t, _)| *t == tag).map(|(_, name)| *name)
    }

    /// Whether `tag` is in the product's reserved list (a retired tag from
    /// an older schema version).
    pub fn is_reserved(self, tag: Tag) -> bool {
        self.reserved.contains(&tag)
    }

    /// Whether the field at `tag` is marked `#[tag(N, default)]` —
    /// wire-tolerant: the decoder fills `T::default()` when it's missing.
    pub fn is_default(self, tag: Tag) -> bool {
        self.defaults.contains(&tag)
    }
}

/// One variant of a sum type. Its payload is a single [`Product`] (possibly
/// with no fields for unit variants).
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub tag: Tag,
    pub name: &'static str,
    pub payload: Product,
}

/// A sum type — a discriminated union of [`Variant`]s. `reserved` lists
/// retired *variant* tags (compile-time-only, prevents reuse).
///
/// Note: there's no `allow_unknown_tags` at the sum level. An unknown variant
/// tag has no `skip` semantics — there's no fragment to skip, since the value
/// itself is structurally unrepresentable when its discriminator is unknown.
#[derive(Clone, Copy, Debug)]
pub struct Sum {
    pub variants: &'static [Variant],
    pub reserved: &'static [Tag],
}

impl Sum {
    /// Look up a variant's metadata by its serde name.
    pub fn variant_for(self, variant_name: &str) -> Option<Variant> {
        self.variants.iter().find(|v| v.name == variant_name).copied()
    }

    /// Whether `tag` is in the sum's reserved list (a retired variant tag).
    pub fn is_reserved(self, tag: Tag) -> bool {
        self.reserved.contains(&tag)
    }
}

/// A registered type's metadata. Stores only the type's [`Tagged`] shape
/// alongside a `TypeId` used to detect serde-name collisions between
/// different Rust types.
#[derive(Debug)]
pub struct Entry {
    type_id: TypeId,
    tagged: Tagged,
}

impl Entry {
    /// The type's wire shape — match on it to dispatch product vs. sum.
    pub fn tagged(&self) -> Tagged {
        self.tagged
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
                slot.insert(Entry { type_id: TypeId::of::<T>(), tagged: T::TAGGED });
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

    /// Hand-written struct-shaped impl exercising every `Product` field.
    struct Foo;
    impl MsgpackTagged for Foo {
        const TAGGED: Tagged = Tagged::Product(Product {
            fields: &[(0, "a"), (1, "b")],
            reserved: &[3],
            defaults: &[1],
            allow_unknown_tags: true,
        });
        fn register_into(_reg: &mut TagRegistry) {}
    }

    /// Hand-written struct with all `Product` extras at their defaults.
    struct Bar;
    impl MsgpackTagged for Bar {
        const TAGGED: Tagged = Tagged::Product(Product {
            fields: &[(0, "x")],
            reserved: &[],
            defaults: &[],
            allow_unknown_tags: false,
        });
        fn register_into(_reg: &mut TagRegistry) {}
    }

    /// Hand-written sum-shaped impl: stand-in for what the derive macro will
    /// emit for `enum Choice { #[tag(0)] Empty, #[tag(1)] Pair { #[tag(0)] a, #[tag(2, default)] b } }`.
    struct Choice;
    impl MsgpackTagged for Choice {
        const TAGGED: Tagged = Tagged::Sum(Sum {
            variants: &[
                Variant {
                    tag: 0,
                    name: "Empty",
                    payload: Product {
                        fields: &[],
                        reserved: &[],
                        defaults: &[],
                        allow_unknown_tags: false,
                    },
                },
                Variant {
                    tag: 1,
                    name: "Pair",
                    payload: Product {
                        fields: &[(0, "a"), (2, "b")],
                        reserved: &[],
                        defaults: &[2],
                        allow_unknown_tags: false,
                    },
                },
            ],
            reserved: &[5],
        });
        fn register_into(_reg: &mut TagRegistry) {}
    }

    fn product_of<T: MsgpackTagged>() -> Product {
        T::TAGGED.as_product().expect("expected a product-shaped type")
    }

    fn sum_of<T: MsgpackTagged>() -> Sum {
        T::TAGGED.as_sum().expect("expected a sum-shaped type")
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
    fn get_returns_entry_with_the_types_tagged_shape() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Foo>("Foo");
        let entry = reg.get("Foo").unwrap();
        let p = entry.tagged().as_product().unwrap();
        assert_eq!(p.fields, &[(0, "a"), (1, "b")]);
        assert_eq!(p.reserved, &[3]);
        assert_eq!(p.defaults, &[1]);
        assert!(p.allow_unknown_tags);
    }

    #[test]
    fn get_returns_none_for_unknown_name() {
        let reg = TagRegistry::new();
        assert!(reg.get("Anything").is_none());
    }

    #[test]
    fn product_tag_for_finds_known_fields() {
        let p = product_of::<Foo>();
        assert_eq!(p.tag_for("a"), Some(0));
        assert_eq!(p.tag_for("b"), Some(1));
        assert_eq!(p.tag_for("missing"), None);
    }

    #[test]
    fn product_field_for_finds_known_tags() {
        let p = product_of::<Foo>();
        assert_eq!(p.field_for(0), Some("a"));
        assert_eq!(p.field_for(1), Some("b"));
        assert_eq!(p.field_for(99), None);
    }

    #[test]
    fn product_is_reserved_only_for_listed_tags() {
        let p = product_of::<Foo>();
        assert!(p.is_reserved(3));
        assert!(!p.is_reserved(0));
        assert!(!p.is_reserved(99));
    }

    #[test]
    fn product_is_default_only_for_listed_tags() {
        let p = product_of::<Foo>();
        assert!(p.is_default(1), "Foo's tag 1 (`b`) is in defaults");
        assert!(!p.is_default(0), "tag 0 (`a`) is not defaulted");
        assert!(!p.is_default(99), "unknown tags are not defaulted");
    }

    #[test]
    fn empty_registry() {
        let reg = TagRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn as_product_returns_none_for_sum_shapes() {
        assert!(<Choice as MsgpackTagged>::TAGGED.as_product().is_none());
    }

    #[test]
    fn as_sum_returns_none_for_product_shapes() {
        assert!(<Foo as MsgpackTagged>::TAGGED.as_sum().is_none());
        assert!(<Bar as MsgpackTagged>::TAGGED.as_sum().is_none());
    }

    #[test]
    fn sum_variants_propagate_from_trait_const_to_entry() {
        let mut reg = TagRegistry::new();
        reg.try_insert::<Choice>("Choice");
        let s = reg.get("Choice").unwrap().tagged().as_sum().unwrap();
        assert_eq!(s.variants.len(), 2);
        assert_eq!(s.variants[0].name, "Empty");
        assert_eq!(s.variants[1].name, "Pair");
    }

    #[test]
    fn sum_variant_for_finds_variant_by_name() {
        let s = sum_of::<Choice>();
        let pair = s.variant_for("Pair").expect("`Pair` variant exists");
        assert_eq!(pair.tag, 1);
        assert_eq!(pair.payload.fields, &[(0, "a"), (2, "b")]);
        assert!(s.variant_for("Missing").is_none());
    }

    #[test]
    fn variant_payload_lookups_resolve_payload_field_tags() {
        let pair = sum_of::<Choice>().variant_for("Pair").unwrap();
        assert_eq!(pair.payload.tag_for("a"), Some(0));
        assert_eq!(pair.payload.tag_for("b"), Some(2));
        assert_eq!(pair.payload.tag_for("missing"), None);
        assert_eq!(pair.payload.field_for(0), Some("a"));
        assert_eq!(pair.payload.field_for(2), Some("b"));
        assert_eq!(pair.payload.field_for(99), None);
    }

    #[test]
    fn variant_payload_is_default_uses_per_variant_defaults_list() {
        let pair = sum_of::<Choice>().variant_for("Pair").unwrap();
        assert!(pair.payload.is_default(2), "tag 2 (`b`) is `#[tag(2, default)]`");
        assert!(!pair.payload.is_default(0), "tag 0 (`a`) is not defaulted");
        assert!(!pair.payload.is_default(99), "unknown tags are not defaulted");
    }

    /// Unit variants have empty `fields` and `defaults` slices — the wrapper
    /// can rely on this to short-circuit field-table lookups.
    #[test]
    #[allow(clippy::const_is_empty)]
    fn unit_variants_have_empty_field_and_default_tables() {
        let empty = sum_of::<Choice>().variant_for("Empty").unwrap();
        assert!(empty.payload.fields.is_empty());
        assert!(empty.payload.defaults.is_empty());
    }

    #[test]
    fn sum_is_reserved_only_for_listed_variant_tags() {
        let s = sum_of::<Choice>();
        assert!(s.is_reserved(5));
        assert!(!s.is_reserved(0));
        assert!(!s.is_reserved(99));
    }
}
