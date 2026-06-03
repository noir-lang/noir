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

    /// Empty [Tagged::Product] used for primitives and _newtypes_.
    pub const fn empty_product() -> Self {
        Self::Product(Product::empty())
    }
}

/// A product type — a fixed list of named, integer-tagged fields. Used for
/// top-level structs/tuple structs *and* for an enum variant's payload (a
/// variant is structurally just a struct hung off a tag).
///
/// `fields` is in tag-ascending order (the canonical wire order). `reserved`
/// lists tags previously used by this product and now retired — purely
/// compile-time metadata that prevents reuse, never affects decode behavior.
/// `allow_unknown_tags` opts the decoder into silently skipping fields whose
/// tag isn't in `fields` or `reserved`. Per-field wire-tolerance (i.e. "fill
/// `T::default()` when this tag is missing") is **not** modeled here — it's
/// expressed on the user side via serde-derive's `#[serde(default)]`, which
/// is what actually performs the substitution at decode time.
///
/// `tag_order_matches_source` says the user's source-declaration order is
/// already tag-ascending — i.e. the order serde-derive will call
/// `serialize_field` in matches the canonical wire order. Set by the macro
/// at derive time. The encoder uses it to skip the buffer-and-sort flush
/// under the `Array` strategy when source order is already correct, saving
/// a per-field `Vec<u8>` allocation. Under `Tagged` the encoder always
/// writes direct (no canonical-byte-order promise), so this flag is
/// unused there.
#[derive(Clone, Copy, Debug)]
pub struct Product {
    pub fields: &'static [(Tag, &'static str)],
    pub reserved: &'static [Tag],
    pub allow_unknown_tags: bool,
    pub tag_order_matches_source: bool,
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

    /// Empty [Product] used for primitives and _newtypes_.
    pub const fn empty() -> Self {
        // No fields ⇒ trivially monotonic (no order to violate).
        Self {
            fields: &[],
            reserved: &[],
            allow_unknown_tags: false,
            tag_order_matches_source: true,
        }
    }
}

/// The shape of an enum variant's payload, used by the wrapper to decide how
/// to encode/decode the value carried under the variant tag.
///
/// `Unit` and `Newtype` both have an empty `payload` `Product` — the
/// distinction lives in this discriminator. A unit variant carries no value
/// at all (the wire emits the variant tag with no payload), while a newtype
/// variant passes the inner value through directly under the variant tag
/// (zero-cost wrapper, no field-level tag/key allocated for the inner value).
///
/// `Tuple` and `Struct` variants both carry their fields in the variant's
/// `payload` `Product`, but differ in addressing on the wire: tuple variants
/// use positional names ("0", "1", …) and struct variants use field idents.
/// Tuple variants with a single explicitly tagged field still count as
/// `Tuple`, not `Newtype` — the explicit `#[tag(N)]` is what asks for a
/// field-level tag wrapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantKind {
    Unit,
    Newtype,
    Tuple,
    Struct,
}

/// One variant of a sum type. Its payload is a single [`Product`] (possibly
/// with no fields for unit and newtype variants — see [`VariantKind`] for the
/// discriminator that distinguishes them).
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub tag: Tag,
    pub name: &'static str,
    pub kind: VariantKind,
    pub payload: Product,
}

/// A sum type — a discriminated union of [`Variant`]s.
///
/// `reserved` lists retired *variant* tags. Like `Product::reserved`, this is
/// always a compile-time tag-reuse guard; whether the runtime decoder routes
/// such tags to a fallback variant is controlled by `on_reserved_tag`.
///
/// `on_reserved_tag` and `on_unknown_tag` opt into runtime-lenient decode of
/// variant tags. Unlike products' `allow_unknown_tags` (which just skips an
/// entry), sums can't skip a discriminator — the value is the discriminator —
/// so the tolerance is expressed as "route to a designated fallback variant,
/// discarding the payload":
///
/// * `on_reserved_tag` — when set, the wire tag of the unit variant that
///   acts as the backward-compat fallback. The macro fills it in iff a
///   variant in the source carries `#[tagged(on_reserved)]`. On decode,
///   any wire tag in `reserved` is routed here (payload discarded).
/// * `on_unknown_tag` — same shape, but for forward-compat: a variant
///   marked `#[tagged(on_unknown)]` catches any wire tag that's neither in
///   `variants` nor in `reserved`. **More dangerous** than `on_reserved`:
///   silently swallows real corruption alongside future-version tags, so
///   opt in only when the fallback variant is a safe semantic substitute
///   for "anything I don't recognize" (e.g. metadata-bearing
///   `InlineType`-shaped types — definitely not `BrilligOpcode`-shaped
///   ones, where an unknown discriminator means we can't execute the
///   program).
///
/// A single variant may carry both `#[tagged(on_reserved)]` and
/// `#[tagged(on_unknown)]` when the user wants the unified-catch-all
/// behavior; in that case both fields point at the same tag.
#[derive(Clone, Copy, Debug)]
pub struct Sum {
    pub variants: &'static [Variant],
    pub reserved: &'static [Tag],
    pub on_reserved_tag: Option<Tag>,
    pub on_unknown_tag: Option<Tag>,
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

    /// The registered Rust type's [`TypeId`]. Used by the serializer to
    /// bridge serde's `&str` name (received at `serialize_struct` time)
    /// to the `TypeId`-keyed per-type strategy overrides.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

/// The basename component of `std::any::type_name::<T>()` — module path
/// stripped, generic parameters dropped. Used by the strategy-override
/// machinery to look up registered types by the same serde name that
/// `#[serde(rename = "...")]` (or the bare type ident) maps to.
///
/// Examples (illustrative — actual results depend on the compiler):
/// * `Circuit<FieldElement>` → `"Circuit"`
/// * `acir::circuit::Program<acir_field::FieldElement>` → `"Program"`
/// * `Vec<u32>` → `"Vec"`
/// * `u32` → `"u32"`
///
/// Caveat: a shadow-DTO type with `#[serde(rename = "Public")]` has
/// `type_name` = `"…::PublicWire"` (the Rust type) but registers under
/// `"Public"` (the serde name). The public type that delegates via
/// `#[tagged(via(PublicWire<F>))]` has `type_name` = `"…::Public"`,
/// which lines up. So passing the public type to
/// `with_strategy::<Public<F>>` works; passing the wire DTO directly
/// (`with_strategy::<PublicWire<F>>`) would silently miss.
pub fn type_name_basename<T: ?Sized>() -> &'static str {
    let full: &'static str = std::any::type_name::<T>();
    // Strip generic parameters: everything from the first `<` onward.
    let no_generics: &'static str = full.split_once('<').map_or(full, |(head, _)| head);
    // Strip module path: everything before and including the last `::`.
    no_generics.rsplit_once("::").map_or(no_generics, |(_, tail)| tail)
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

    /// Construct a registry by starting the type-graph walk at `T`. Calls
    /// `T::register_into` against a fresh registry, which registers `T`
    /// itself and then recurses through every reachable tagged field/variant
    /// type. The standard one-shot way to build a registry for a top-level
    /// value about to be encoded.
    ///
    /// ```ignore
    /// let registry = TagRegistry::from_type::<Program>();
    /// ```
    pub fn from_type<T: ?Sized + MsgpackTagged>() -> Self {
        let mut reg = Self::new();
        T::register_into(&mut reg);
        reg
    }

    /// Whether `name` corresponds to a registered serde name. Used by
    /// [`crate::Serializer::with_strategy`] to fail fast when a strategy
    /// override targets a name the registry never saw — almost always a
    /// type-graph miss bug. Pair with [`type_name_basename`] when starting
    /// from a Rust type:
    ///
    /// ```ignore
    /// registry.contains(type_name_basename::<Circuit<F>>())
    /// ```
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
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
            allow_unknown_tags: true,
            tag_order_matches_source: true,
        });
        fn register_into(_reg: &mut TagRegistry) {}
    }

    /// Hand-written struct with all `Product` extras at their defaults.
    struct Bar;
    impl MsgpackTagged for Bar {
        const TAGGED: Tagged = Tagged::Product(Product {
            fields: &[(0, "x")],
            reserved: &[],
            allow_unknown_tags: false,
            tag_order_matches_source: true,
        });
        fn register_into(_reg: &mut TagRegistry) {}
    }

    /// Hand-written sum-shaped impl: stand-in for what the derive macro will
    /// emit for `enum Choice { #[tag(0)] Empty, #[tag(1)] Pair { #[tag(0)] a, #[tag(2)] b } }`.
    struct Choice;
    impl MsgpackTagged for Choice {
        const TAGGED: Tagged = Tagged::Sum(Sum {
            variants: &[
                Variant {
                    tag: 0,
                    name: "Empty",
                    kind: VariantKind::Unit,
                    payload: Product::empty(),
                },
                Variant {
                    tag: 1,
                    name: "Pair",
                    kind: VariantKind::Struct,
                    payload: Product {
                        fields: &[(0, "a"), (2, "b")],
                        reserved: &[],
                        allow_unknown_tags: false,
                        tag_order_matches_source: true,
                    },
                },
            ],
            reserved: &[5],
            on_reserved_tag: None,
            on_unknown_tag: None,
        });
        fn register_into(_reg: &mut TagRegistry) {}
    }

    /// Hand-written sum exercising both fallback markers together. Mirrors
    /// the derive-macro emission for an enum like
    /// `#[tagged(reserved(7))] enum Lenient { #[tag(0)] A, #[tag(1)] B, #[tag(2)] #[tagged(on_reserved, on_unknown)] Other }`.
    struct Lenient;
    impl MsgpackTagged for Lenient {
        const TAGGED: Tagged = Tagged::Sum(Sum {
            variants: &[
                Variant { tag: 0, name: "A", kind: VariantKind::Unit, payload: Product::empty() },
                Variant { tag: 1, name: "B", kind: VariantKind::Unit, payload: Product::empty() },
                Variant {
                    tag: 2,
                    name: "Other",
                    kind: VariantKind::Unit,
                    payload: Product::empty(),
                },
            ],
            reserved: &[7],
            on_reserved_tag: Some(2),
            on_unknown_tag: Some(2),
        });
        fn register_into(_reg: &mut TagRegistry) {}
    }

    fn product_of<T: MsgpackTagged>() -> Product {
        T::TAGGED.as_product().expect("expected a product-shaped type")
    }

    fn sum_of<T: MsgpackTagged>() -> Sum {
        T::TAGGED.as_sum().expect("expected a sum-shaped type")
    }

    /// Self-registering fixture: unlike `Foo` / `Choice`, this fixture's
    /// `register_into` actually populates the registry — exercises the
    /// `TagRegistry::of::<T>` helper end-to-end.
    struct SelfRegistering;
    impl MsgpackTagged for SelfRegistering {
        const TAGGED: Tagged = Tagged::empty_product();
        fn register_into(reg: &mut TagRegistry) {
            reg.try_insert::<Self>("SelfRegistering");
        }
    }

    #[test]
    fn from_type_walks_the_type_graph_from_a_typed_entry_point() {
        let reg = TagRegistry::from_type::<SelfRegistering>();
        assert!(
            reg.get("SelfRegistering").is_some(),
            "SelfRegistering's `register_into` should run via `TagRegistry::from_type`",
        );
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

    /// Unit variants have an empty `fields` slice — the wrapper can rely
    /// on this to short-circuit field-table lookups.
    #[test]
    #[allow(clippy::const_is_empty)]
    fn unit_variants_have_empty_field_table() {
        let empty = sum_of::<Choice>().variant_for("Empty").unwrap();
        assert!(empty.payload.fields.is_empty());
    }

    #[test]
    fn sum_is_reserved_only_for_listed_variant_tags() {
        let s = sum_of::<Choice>();
        assert!(s.is_reserved(5));
        assert!(!s.is_reserved(0));
        assert!(!s.is_reserved(99));
    }

    /// Both fallback-tag slots default to `None` — strict decode unless
    /// the type opts in via a variant-level marker.
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn sum_default_decode_policy_is_strict() {
        let s = sum_of::<Choice>();
        assert!(s.on_reserved_tag.is_none());
        assert!(s.on_unknown_tag.is_none());
    }

    #[test]
    fn sum_decode_policy_flags_propagate_when_set() {
        let s = sum_of::<Lenient>();
        assert_eq!(s.on_reserved_tag, Some(2));
        assert_eq!(s.on_unknown_tag, Some(2));
    }
}
