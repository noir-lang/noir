//! End-to-end tests for the `MsgpackTagged` derive on structs and enums.
//!
//! Unit structs still fall through to the stub expansion (empty `TAGS`, no-op
//! `register_into`); their tests here just verify the derive produces a valid
//! impl. As the macro learns to handle each shape, the corresponding tests
//! in this file get tightened.

// Test fixtures only exist to feed the derive; unused fields are expected.
#![allow(dead_code)]

use msgpack_tagged::{MsgpackTagged, TagRegistry};

#[derive(MsgpackTagged)]
struct Unit;

/// Multi-element tuple struct with implicit positional tags (0, 1).
#[derive(MsgpackTagged)]
struct Tuple(u32, bool);

/// Multi-element tuple struct with explicit per-field `#[tag(N)]`. Field
/// positions are reordered relative to tag order — proves the macro sorts
/// `TAGS` by tag value, not source position.
#[derive(MsgpackTagged)]
struct ExplicitTuple(#[tag(3, default)] u32, #[tag(0)] bool, #[tag(1)] u8);

/// Newtype (single-element tuple struct). Wire bytes are the inner u32's
/// bytes; the newtype itself doesn't appear in the registry.
#[derive(MsgpackTagged)]
struct Witness(u32);

/// Generic newtype, exercising the bound chain through the inner type.
/// `Wrapper<Inner>` reaches `Inner::register_into` via the `where` clause
/// without registering `Wrapper` itself.
#[derive(MsgpackTagged)]
struct Wrapper<T>(T);

#[derive(MsgpackTagged)]
struct Named {
    #[tag(0)]
    a: u32,
    #[tag(1)]
    b: bool,
}

/// Enum mixing all three variant shapes — unit, tuple, struct — to prove the
/// derive handles each. Variant tags are out of declaration order to verify
/// the canonical tag-ascending ordering on `TAGS`.
#[derive(MsgpackTagged)]
enum Choice {
    #[tag(2)]
    Multi { a: u32, b: bool },
    #[tag(0)]
    Nothing,
    #[tag(1)]
    Single(u32),
}

/// Enum whose variant payloads include a `MsgpackTagged` user type, used to
/// verify `register_into` recurses into payload types.
#[derive(MsgpackTagged)]
enum WithInnerPayload {
    #[tag(0)]
    Empty,
    #[tag(1)]
    OneInner(Inner),
    #[tag(2)]
    Pair { left: Inner, right: bool },
}

/// Generic enum exercising the per-payload-type `where` bound on enums.
/// `GenericChoice<Inner>` reaches `Inner::register_into` via the bound chain
/// without the macro needing to know `T`'s identity.
#[derive(MsgpackTagged)]
enum GenericChoice<T> {
    #[tag(0)]
    Nothing,
    #[tag(1)]
    Some(T),
}

/// Enum with `#[tagged(reserved(...))]`: tags 1 and 2 are retired and must
/// never be reused for a future variant.
#[derive(MsgpackTagged)]
#[tagged(reserved(1, 2))]
enum WithReservedVariants {
    #[tag(0)]
    First,
    #[tag(3)]
    Fourth,
}

/// Variant payloads of `PhantomData<T>` are auto-skipped, just like in
/// structs — the bound chain doesn't reach `T`, so non-`MsgpackTagged` types
/// like `Opaque` can still be plugged in.
#[derive(MsgpackTagged)]
enum WithPhantomVariant<T> {
    #[tag(0)]
    Empty,
    #[tag(1)]
    Tagged(std::marker::PhantomData<T>),
    #[tag(2)]
    Named { _marker: std::marker::PhantomData<T> },
}

/// Inner type that registers itself, used to verify recursion through
/// `register_into` lands in the registry.
#[derive(MsgpackTagged)]
struct Inner {
    #[tag(0)]
    x: u32,
}

#[derive(MsgpackTagged)]
struct Outer {
    #[tag(0)]
    inner: Inner,
    #[tag(1)]
    flag: bool,
}

#[derive(MsgpackTagged)]
struct Generic<T> {
    #[tag(0)]
    value: T,
    #[tag(1)]
    count: u32,
}

/// `WithMap<K, V>` exercises the per-field-type `where` bound: the impl emits
/// `where BTreeMap<K, V>: MsgpackTagged` rather than `K: MsgpackTagged, V: MsgpackTagged`,
/// so whatever bounds `BTreeMap`'s impl requires get propagated transitively.
#[derive(MsgpackTagged)]
struct WithMap<K, V> {
    #[tag(0)]
    map: std::collections::BTreeMap<K, V>,
}

/// Multiple fields of the same type — exercises dedup in the where-clause
/// builder so we don't emit `u32: MsgpackTagged` twice.
#[derive(MsgpackTagged)]
struct SameTypeFields {
    #[tag(0)]
    a: u32,
    #[tag(1)]
    b: u32,
    #[tag(2)]
    c: u32,
}

/// Tags are not declared in source order, to assert the canonical
/// tag-ascending ordering the derive should produce.
#[derive(MsgpackTagged)]
struct OutOfOrder {
    #[tag(2)]
    c: u32,
    #[tag(1)]
    a: u32,
    #[tag(0)]
    b: u32,
}

/// Type that intentionally does *not* implement `MsgpackTagged`, used to
/// verify that `#[tag(skip)]` and `PhantomData<_>` exempt their field type
/// from the bound chain.
struct Opaque {
    payload: Vec<u8>,
}

/// `#[tag(skip)]` on a field whose type isn't `MsgpackTagged`. The container
/// still derives because the skipped field doesn't contribute a where bound.
#[derive(MsgpackTagged)]
struct WithExplicitSkip {
    #[tag(0)]
    visible: u32,
    #[tag(skip)]
    hidden: Opaque,
}

/// `PhantomData<T>` auto-skip: no `#[tag]` annotation needed, and the
/// container-impl works for `T` without requiring `T: MsgpackTagged`.
#[derive(MsgpackTagged)]
struct WithPhantom<T> {
    #[tag(0)]
    visible: u32,
    _phantom: std::marker::PhantomData<T>,
}

/// `#[tag(N, default)]` fields: tag 1 (`extra`) is wire-tolerant — appears in
/// `TAGS` and `DEFAULTS`, decoder will fill `Vec::default()` if missing.
#[derive(MsgpackTagged)]
struct WithDefaults {
    #[tag(0)]
    required: u32,
    #[tag(1, default)]
    extra: Vec<u8>,
    #[tag(2, default)]
    annotation: String,
}

/// Type-level `#[tagged(reserved(...))]`: tags 1 and 4 have been retired and must
/// never be reused. The macro emits these into `RESERVED`, and a `#[tag(1)]`
/// or `#[tag(4)]` on any field would now be a compile error.
#[derive(MsgpackTagged)]
#[tagged(reserved(1, 4))]
struct WithReserved {
    #[tag(0)]
    a: u32,
    #[tag(2)]
    b: u32,
    #[tag(3)]
    c: u32,
}

/// Type-level `#[tagged(allow_unknown_tags)]`: opts the type into lenient decode of
/// unknown tags. Recommended for top-level metadata-bearing types like
/// `Program` and `Circuit`; not for cryptographic-shape types where silently
/// dropping fields could change proof semantics.
#[derive(MsgpackTagged)]
#[tagged(allow_unknown_tags)]
struct LenientType {
    #[tag(0)]
    a: u32,
}

/// Wire DTO that registers itself; the `Public` type below delegates to this
/// via `#[tagged(via(...))]`.
#[derive(MsgpackTagged)]
struct WireDto {
    #[tag(0)]
    payload: u32,
}

/// Realistic shadow-DTO setup: the wire type carries `#[serde(rename = "...")]`
/// pointing at the public type's name, so on the wire it appears as if the
/// public type were being serialized directly. The macro reads the `rename`
/// and uses it as the registry key — matching what `serialize_struct` will
/// pass at runtime through the auto-derived `Serialize` impl.
#[derive(MsgpackTagged, serde::Serialize)]
#[serde(rename = "Renamed")]
struct RenamedWire {
    #[tag(0)]
    payload: u32,
}

/// `#[tagged(via(WireDto))]`: the macro emits a delegation impl that calls
/// `WireDto::register_into` instead of registering `Public` itself. The
/// public type's own fields are wire-irrelevant — they must NOT carry
/// `#[tag(...)]` annotations (the macro rejects that combination — see the
/// `via_with_field_tag` compile-fail test), and they don't constrain anything.
#[derive(MsgpackTagged)]
#[tagged(via(WireDto))]
struct Public {
    internal_only: Opaque,
    other: Vec<u8>,
}

#[test]
fn derive_compiles_for_basic_shapes() {
    fn assert_impl<T: MsgpackTagged>() {}
    assert_impl::<Unit>();
    assert_impl::<Tuple>();
    assert_impl::<ExplicitTuple>();
    assert_impl::<Witness>();
    assert_impl::<Wrapper<Inner>>();
    assert_impl::<Named>();
    assert_impl::<Choice>();
    assert_impl::<WithInnerPayload>();
    assert_impl::<GenericChoice<u32>>();
    assert_impl::<GenericChoice<Inner>>();
    assert_impl::<WithReservedVariants>();
    // T = Opaque (no MsgpackTagged impl) still satisfies the enum's bounds,
    // because PhantomData<T> in variant payloads is auto-skipped.
    assert_impl::<WithPhantomVariant<Opaque>>();
    assert_impl::<Inner>();
    assert_impl::<Outer>();
    assert_impl::<Generic<u32>>();
    assert_impl::<Generic<Inner>>();
    assert_impl::<OutOfOrder>();
    assert_impl::<WithMap<u32, Inner>>();
    assert_impl::<SameTypeFields>();
    assert_impl::<WithExplicitSkip>();
    // T = Opaque (no MsgpackTagged impl) still satisfies WithPhantom<T>'s bound,
    // because PhantomData<T> is auto-skipped — the bound chain doesn't reach T.
    assert_impl::<WithPhantom<Opaque>>();
    assert_impl::<WithDefaults>();
    assert_impl::<WithReserved>();
    assert_impl::<LenientType>();
    assert_impl::<WireDto>();
    assert_impl::<Public>();
    assert_impl::<RenamedWire>();
}

#[test]
fn unit_struct_has_empty_tags() {
    assert!(<Unit as MsgpackTagged>::TAGS.is_empty());
}

#[test]
fn implicit_tuple_struct_uses_positional_tags() {
    assert_eq!(<Tuple as MsgpackTagged>::TAGS, &[(0, "0"), (1, "1")]);
}

#[test]
fn explicit_tuple_struct_tags_match_annotations_and_sort_by_tag() {
    // Source: (#[tag(3, default)] u32, #[tag(0)] bool, #[tag(1)] u8)
    // After tag-ascending sort: position-string names follow the tags.
    assert_eq!(<ExplicitTuple as MsgpackTagged>::TAGS, &[(0, "1"), (1, "2"), (3, "0")]);
    assert_eq!(<ExplicitTuple as MsgpackTagged>::DEFAULTS, &[3]);
}

#[test]
fn tuple_struct_register_into_populates_registry() {
    let mut reg = TagRegistry::new();
    Tuple::register_into(&mut reg);
    assert!(reg.get("Tuple").is_some(), "tuple structs register themselves");
}

#[test]
fn newtype_does_not_register_itself_but_recurses_into_inner() {
    let mut reg = TagRegistry::new();
    Wrapper::<Inner>::register_into(&mut reg);
    assert!(reg.get("Wrapper").is_none(), "newtype passes through; no registry entry of its own");
    assert!(reg.get("Inner").is_some(), "the inner type is reached via the bound chain");
}

#[test]
fn newtype_constants_are_empty() {
    assert!(<Witness as MsgpackTagged>::TAGS.is_empty());
    assert!(<Witness as MsgpackTagged>::RESERVED.is_empty());
    assert!(<Witness as MsgpackTagged>::DEFAULTS.is_empty());
}

#[test]
fn named_struct_tags_match_declarations() {
    assert_eq!(<Named as MsgpackTagged>::TAGS, &[(0, "a"), (1, "b")]);
}

#[test]
fn tags_are_emitted_in_tag_order_not_source_order() {
    assert_eq!(<OutOfOrder as MsgpackTagged>::TAGS, &[(0, "b"), (1, "a"), (2, "c")]);
}

#[test]
fn named_struct_register_into_populates_registry_under_type_name() {
    let mut reg = TagRegistry::new();
    Named::register_into(&mut reg);
    let entry = reg.get("Named").expect("Named should register itself");
    assert_eq!(entry.tags(), &[(0, "a"), (1, "b")]);
}

/// Idempotent re-registration: calling `register_into` twice produces a
/// registry with one entry, the second call short-circuits.
#[test]
fn named_struct_register_into_is_idempotent() {
    let mut reg = TagRegistry::new();
    Named::register_into(&mut reg);
    Named::register_into(&mut reg);
    assert_eq!(reg.len(), 1);
}

#[test]
fn nested_register_into_walks_the_field_graph() {
    let mut reg = TagRegistry::new();
    Outer::register_into(&mut reg);
    assert!(reg.get("Outer").is_some(), "Outer registers itself");
    assert!(reg.get("Inner").is_some(), "register_into recurses into field types");
}

/// `Generic<T>` reaches `T`'s `register_into` via the bound chain. Pass
/// `Inner` as `T` and verify it ends up in the registry.
#[test]
fn generic_struct_recurses_into_its_concrete_type_parameter() {
    let mut reg = TagRegistry::new();
    <Generic<Inner>>::register_into(&mut reg);
    assert!(reg.get("Generic").is_some());
    assert!(reg.get("Inner").is_some());
}

/// The per-field-type `where` bound transitively propagates whatever the
/// inner container's impl requires. `WithMap<u32, Inner>` works because
/// `BTreeMap<u32, Inner>: MsgpackTagged` holds (its blanket impl needs both
/// key and value to implement the trait, and they do).
#[test]
fn generic_struct_with_container_field_recurses_into_both_inner_types() {
    let mut reg = TagRegistry::new();
    <WithMap<u32, Inner>>::register_into(&mut reg);
    assert!(reg.get("WithMap").is_some());
    assert!(
        reg.get("Inner").is_some(),
        "BTreeMap value type should be reached via the bound chain"
    );
}

#[test]
fn explicit_skip_field_is_absent_from_tags() {
    assert_eq!(<WithExplicitSkip as MsgpackTagged>::TAGS, &[(0, "visible")]);
}

#[test]
fn phantom_data_field_is_absent_from_tags() {
    assert_eq!(<WithPhantom<Opaque> as MsgpackTagged>::TAGS, &[(0, "visible")]);
}

#[test]
fn default_fields_appear_in_both_tags_and_defaults() {
    assert_eq!(
        <WithDefaults as MsgpackTagged>::TAGS,
        &[(0, "required"), (1, "extra"), (2, "annotation")],
        "default fields still appear in TAGS — they're encoded normally, only the decoder is tolerant",
    );
    assert_eq!(
        <WithDefaults as MsgpackTagged>::DEFAULTS,
        &[1, 2],
        "DEFAULTS lists exactly the tags marked `#[tag(N, default)]`",
    );
}

#[test]
fn defaults_show_up_on_the_registry_entry() {
    let mut reg = TagRegistry::new();
    WithDefaults::register_into(&mut reg);
    let entry = reg.get("WithDefaults").expect("WithDefaults should register itself");
    assert!(!entry.is_default(0), "tag 0 (`required`) is not defaulted");
    assert!(entry.is_default(1), "tag 1 (`extra`) is defaulted");
    assert!(entry.is_default(2), "tag 2 (`annotation`) is defaulted");
}

#[test]
fn reserved_tags_appear_in_the_const_and_registry() {
    assert_eq!(<WithReserved as MsgpackTagged>::RESERVED, &[1, 4]);

    let mut reg = TagRegistry::new();
    WithReserved::register_into(&mut reg);
    let entry = reg.get("WithReserved").expect("WithReserved should register itself");
    assert!(entry.is_reserved(1));
    assert!(entry.is_reserved(4));
    assert!(!entry.is_reserved(0));
    assert!(!entry.is_reserved(2));
}

#[test]
fn reserved_tags_do_not_appear_in_tags() {
    assert_eq!(<WithReserved as MsgpackTagged>::TAGS, &[(0, "a"), (2, "b"), (3, "c")]);
}

/// Verifies the `#[tagged(allow_unknown_tags)]` attribute flips the trait const, and
/// the absence of the attribute leaves the default `false` in place. The
/// `#[allow]` is needed because the assertion's truth is statically known —
/// intentional, the test catches a regression if the macro stops emitting the
/// attribute-driven value.
#[test]
#[allow(clippy::assertions_on_constants)]
fn allow_unknown_tags_flag_is_propagated() {
    assert!(<LenientType as MsgpackTagged>::ALLOW_UNKNOWN_TAGS);
    // Default for any other type — verified here via a fixture without the attr.
    assert!(!<Named as MsgpackTagged>::ALLOW_UNKNOWN_TAGS);
}

#[test]
fn allow_unknown_tags_shows_up_on_the_registry_entry() {
    let mut reg = TagRegistry::new();
    LenientType::register_into(&mut reg);
    let entry = reg.get("LenientType").expect("LenientType should register itself");
    assert!(entry.allow_unknown_tags());
}

/// `via`-delegating type doesn't put itself in the registry; instead, calling
/// its `register_into` registers the wire DTO under the wire DTO's name.
///
/// Without `#[serde(rename)]` the wire DTO registers under its Rust ident
/// (`"WireDto"`). In real use this would mismatch what `serialize_struct`
/// passes at runtime through the public type's `Serialize` impl — see
/// `serde_rename_drives_registry_key_not_rust_ident` for the realistic case
/// where the wire type uses `#[serde(rename = "Public")]` to align names.
#[test]
fn via_delegates_register_into_to_the_wire_dto() {
    let mut reg = TagRegistry::new();
    Public::register_into(&mut reg);
    assert!(reg.get("WireDto").is_some(), "wire DTO should be registered");
    assert!(reg.get("Public").is_none(), "public type should NOT appear in the registry");
}

/// The public type's own constants are inert (empty / false) — the wire
/// shape comes from the wire DTO, which has its own non-empty TAGS.
#[test]
fn via_public_type_constants_are_empty() {
    assert!(<Public as MsgpackTagged>::TAGS.is_empty());
    assert!(<Public as MsgpackTagged>::RESERVED.is_empty());
    assert!(<Public as MsgpackTagged>::DEFAULTS.is_empty());
    // The wire DTO's TAGS, by contrast, are populated from its own #[tag(N)].
    assert_eq!(<WireDto as MsgpackTagged>::TAGS, &[(0, "payload")]);
}

/// `#[serde(rename = "X")]` on the wire DTO drives the registry key — the
/// wire type registers under its rename target, not its Rust ident. This is
/// the load-bearing mechanism for the shadow-DTO pattern: the wrapper
/// Serializer's `serialize_struct(...)` call (driven by serde's auto-derived
/// `Serialize`) passes the rename target as the name, and the registry
/// lookup matches.
#[test]
fn serde_rename_drives_registry_key_not_rust_ident() {
    let mut reg = TagRegistry::new();
    RenamedWire::register_into(&mut reg);
    assert!(
        reg.get("Renamed").is_some(),
        "registry should be keyed by the `#[serde(rename)]` target"
    );
    assert!(
        reg.get("RenamedWire").is_none(),
        "Rust ident should NOT appear when a rename is present"
    );
}

#[test]
fn enum_tags_are_emitted_in_tag_order_with_variant_names() {
    // Source: Multi (#[tag(2)]), Nothing (#[tag(0)]), Single (#[tag(1)]).
    // After tag-ascending sort, the variant idents land in tag order.
    assert_eq!(<Choice as MsgpackTagged>::TAGS, &[(0, "Nothing"), (1, "Single"), (2, "Multi")],);
}

#[test]
fn enum_register_into_populates_registry_under_type_name() {
    let mut reg = TagRegistry::new();
    Choice::register_into(&mut reg);
    let entry = reg.get("Choice").expect("Choice should register itself");
    assert_eq!(entry.tags(), &[(0, "Nothing"), (1, "Single"), (2, "Multi")]);
}

/// Enums with no payload types still register themselves; the recursion list
/// is empty so no other entries appear.
#[test]
fn enum_register_into_is_idempotent() {
    let mut reg = TagRegistry::new();
    Choice::register_into(&mut reg);
    Choice::register_into(&mut reg);
    assert_eq!(reg.len(), 1);
}

/// `register_into` walks every variant payload — `Inner` is reached through
/// both the tuple and struct variants of `WithInnerPayload`.
#[test]
fn enum_register_into_recurses_into_variant_payloads() {
    let mut reg = TagRegistry::new();
    WithInnerPayload::register_into(&mut reg);
    assert!(reg.get("WithInnerPayload").is_some(), "enum registers itself");
    assert!(
        reg.get("Inner").is_some(),
        "register_into should recurse into the variants' payload types",
    );
}

/// Generic enums hit `T`'s `register_into` via the bound chain, just like
/// generic structs do.
#[test]
fn generic_enum_recurses_into_concrete_payload_type() {
    let mut reg = TagRegistry::new();
    <GenericChoice<Inner>>::register_into(&mut reg);
    assert!(reg.get("GenericChoice").is_some());
    assert!(reg.get("Inner").is_some());
}

#[test]
fn enum_reserved_tags_appear_in_const_and_registry() {
    assert_eq!(<WithReservedVariants as MsgpackTagged>::RESERVED, &[1, 2]);

    let mut reg = TagRegistry::new();
    WithReservedVariants::register_into(&mut reg);
    let entry = reg.get("WithReservedVariants").expect("should register itself");
    assert!(entry.is_reserved(1));
    assert!(entry.is_reserved(2));
    assert!(!entry.is_reserved(0));
    assert!(!entry.is_reserved(3));
}

/// Enums never carry `default` semantics — `DEFAULTS` is always empty.
#[test]
#[allow(clippy::const_is_empty)]
fn enum_defaults_are_always_empty() {
    assert!(<Choice as MsgpackTagged>::DEFAULTS.is_empty());
    assert!(<WithReservedVariants as MsgpackTagged>::DEFAULTS.is_empty());
}
