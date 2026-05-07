//! End-to-end tests for the `MsgpackTagged` derive on structs and enums.
//!
//! Unit structs still fall through to the stub expansion (empty `Tagged::Product`,
//! no-op `register_into`); their tests here just verify the derive produces a
//! valid impl. As the macro learns to handle each shape, the corresponding
//! tests in this file get tightened.

// Test fixtures only exist to feed the derive; unused fields are expected.
#![allow(dead_code)]

use msgpack_tagged::{Entry, MsgpackTagged, Product, Sum, TagRegistry};

#[derive(MsgpackTagged)]
struct Unit;

/// Multi-element tuple struct with implicit positional tags (0, 1).
#[derive(MsgpackTagged)]
struct Tuple(u32, bool);

/// Multi-element tuple struct with explicit per-field `#[tag(N)]`. Field
/// positions are reordered relative to tag order — proves the macro sorts
/// fields by tag value, not source position.
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
/// the canonical tag-ascending ordering on the emitted `Sum.variants`.
/// Named variants need explicit `#[tag(N)]` on every payload field; the
/// single-element `Single` tuple variant takes implicit positional tags.
#[derive(MsgpackTagged)]
enum Choice {
    #[tag(2)]
    Multi {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    },
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
    Pair {
        #[tag(0)]
        left: Inner,
        #[tag(1)]
        right: bool,
    },
}

/// Tuple variant with explicit per-field tags — proves the same all-explicit
/// machinery used by top-level tuple structs works inside variants. Tags are
/// declared out of source order to verify the variant payload's `fields`
/// land in tag-ascending order.
#[derive(MsgpackTagged)]
enum ExplicitTupleVariants {
    #[tag(0)]
    Triple(#[tag(2)] u32, #[tag(0)] bool, #[tag(1, default)] u8),
    #[tag(1)]
    ImplicitPair(u32, bool),
}

/// Named variant exercising the field-level `default` modifier inside a
/// variant payload. The variant-payload field bound is `Vec<u8>: Default`,
/// added by the macro to the impl's where clause. `_phantom: PhantomData<T>`
/// auto-skips and `hidden: Opaque` opts out via `#[tag(skip)]` — neither
/// contributes a `MsgpackTagged` bound, which is why `T = Opaque` works.
#[derive(MsgpackTagged)]
enum WithVariantFieldExtras<T> {
    #[tag(0)]
    Plain {
        #[tag(0)]
        required: u32,
        #[tag(1, default)]
        annotation: Vec<u8>,
        #[tag(skip)]
        hidden: Opaque,
        _phantom: std::marker::PhantomData<T>,
    },
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

/// Enum opting into `default_on_reserved` — encountering a reserved variant
/// tag on decode should produce `Self::default()` instead of erroring. The
/// macro's `where Self: Default` bound is what forces `#[derive(Default)]`
/// here: drop the derive and the impl stops compiling.
#[derive(MsgpackTagged, Default)]
#[tagged(reserved(2), default_on_reserved)]
enum BackwardsCompat {
    #[default]
    #[tag(0)]
    First,
    #[tag(1)]
    Second,
}

/// Enum opting into `default_on_unknown` — forward-compat for types where
/// "I don't recognize this tag" is safely interpretable as `default()`.
#[derive(MsgpackTagged, Default)]
#[tagged(default_on_unknown)]
enum ForwardCompat {
    #[default]
    #[tag(0)]
    Empty,
    #[tag(1)]
    Some(u32),
}

/// Both flags at once — `InlineType`-shaped fully-lenient policy.
#[derive(MsgpackTagged, Default)]
#[tagged(reserved(7), default_on_reserved, default_on_unknown)]
enum FullyLenient {
    #[default]
    #[tag(0)]
    A,
    #[tag(1)]
    B,
}

/// Variant payloads of `PhantomData<T>` rely on the blanket
/// `impl<T: 'static> MsgpackTagged for PhantomData<T>` — its bound is
/// `T: 'static`, not `T: MsgpackTagged`, so non-`MsgpackTagged` types like
/// `Opaque` can still appear behind it.
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
/// `Product.fields` and `Product.defaults`, decoder will fill `Vec::default()`
/// if missing.
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
/// never be reused. The macro emits these into the product's `reserved`, and a
/// `#[tag(1)]` or `#[tag(4)]` on any field would now be a compile error.
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

fn product_of<T: MsgpackTagged>() -> Product {
    T::TAGGED.as_product().expect("expected a product-shaped type")
}

fn sum_of<T: MsgpackTagged>() -> Sum {
    T::TAGGED.as_sum().expect("expected a sum-shaped type")
}

fn entry_product(entry: &Entry) -> Product {
    entry.tagged().as_product().expect("expected entry with product shape")
}

fn entry_sum(entry: &Entry) -> Sum {
    entry.tagged().as_sum().expect("expected entry with sum shape")
}

fn variant_pairs(s: Sum) -> Vec<(u8, &'static str)> {
    s.variants.iter().map(|v| (v.tag, v.name)).collect()
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
    assert_impl::<ExplicitTupleVariants>();
    // T = Opaque (no MsgpackTagged impl) still satisfies the bounds because
    // the `Opaque` field is `#[tag(skip)]` and the `PhantomData<T>` field
    // auto-skips — neither contributes a `MsgpackTagged: Opaque` bound.
    assert_impl::<WithVariantFieldExtras<Opaque>>();
    assert_impl::<GenericChoice<u32>>();
    assert_impl::<GenericChoice<Inner>>();
    assert_impl::<WithReservedVariants>();
    assert_impl::<BackwardsCompat>();
    assert_impl::<ForwardCompat>();
    assert_impl::<FullyLenient>();
    // T = Opaque (no MsgpackTagged impl) still satisfies the enum's bounds —
    // `PhantomData<T>: MsgpackTagged` is blanket-implemented with only
    // `T: 'static`, not `T: MsgpackTagged`.
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
fn unit_struct_has_empty_fields() {
    assert!(product_of::<Unit>().fields.is_empty());
}

#[test]
fn implicit_tuple_struct_uses_positional_tags() {
    assert_eq!(product_of::<Tuple>().fields, &[(0, "0"), (1, "1")]);
}

#[test]
fn explicit_tuple_struct_tags_match_annotations_and_sort_by_tag() {
    // Source: (#[tag(3, default)] u32, #[tag(0)] bool, #[tag(1)] u8)
    // After tag-ascending sort: position-string names follow the tags.
    let p = product_of::<ExplicitTuple>();
    assert_eq!(p.fields, &[(0, "1"), (1, "2"), (3, "0")]);
    assert_eq!(p.defaults, &[3]);
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
    let p = product_of::<Witness>();
    assert!(p.fields.is_empty());
    assert!(p.reserved.is_empty());
    assert!(p.defaults.is_empty());
}

#[test]
fn named_struct_tags_match_declarations() {
    assert_eq!(product_of::<Named>().fields, &[(0, "a"), (1, "b")]);
}

#[test]
fn tags_are_emitted_in_tag_order_not_source_order() {
    assert_eq!(product_of::<OutOfOrder>().fields, &[(0, "b"), (1, "a"), (2, "c")]);
}

#[test]
fn named_struct_register_into_populates_registry_under_type_name() {
    let mut reg = TagRegistry::new();
    Named::register_into(&mut reg);
    let entry = reg.get("Named").expect("Named should register itself");
    assert_eq!(entry_product(entry).fields, &[(0, "a"), (1, "b")]);
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
fn explicit_skip_field_is_absent_from_fields() {
    assert_eq!(product_of::<WithExplicitSkip>().fields, &[(0, "visible")]);
}

#[test]
fn phantom_data_field_is_absent_from_fields() {
    assert_eq!(product_of::<WithPhantom<Opaque>>().fields, &[(0, "visible")]);
}

#[test]
fn default_fields_appear_in_both_fields_and_defaults() {
    let p = product_of::<WithDefaults>();
    assert_eq!(
        p.fields,
        &[(0, "required"), (1, "extra"), (2, "annotation")],
        "default fields still appear on the wire — they're encoded normally, only the decoder is tolerant",
    );
    assert_eq!(p.defaults, &[1, 2], "defaults lists exactly the tags marked `#[tag(N, default)]`",);
}

#[test]
fn defaults_show_up_on_the_registry_entry() {
    let mut reg = TagRegistry::new();
    WithDefaults::register_into(&mut reg);
    let entry = reg.get("WithDefaults").expect("WithDefaults should register itself");
    let p = entry_product(entry);
    assert!(!p.is_default(0), "tag 0 (`required`) is not defaulted");
    assert!(p.is_default(1), "tag 1 (`extra`) is defaulted");
    assert!(p.is_default(2), "tag 2 (`annotation`) is defaulted");
}

#[test]
fn reserved_tags_appear_in_the_const_and_registry() {
    assert_eq!(product_of::<WithReserved>().reserved, &[1, 4]);

    let mut reg = TagRegistry::new();
    WithReserved::register_into(&mut reg);
    let entry = reg.get("WithReserved").expect("WithReserved should register itself");
    let p = entry_product(entry);
    assert!(p.is_reserved(1));
    assert!(p.is_reserved(4));
    assert!(!p.is_reserved(0));
    assert!(!p.is_reserved(2));
}

#[test]
fn reserved_tags_do_not_appear_in_fields() {
    assert_eq!(product_of::<WithReserved>().fields, &[(0, "a"), (2, "b"), (3, "c")]);
}

/// The `#[tagged(allow_unknown_tags)]` attribute flips the product's
/// `allow_unknown_tags` flag, while its absence leaves the default `false`
/// in place.
#[test]
fn allow_unknown_tags_flag_is_propagated() {
    assert!(product_of::<LenientType>().allow_unknown_tags);
    // Default for any other type — verified here via a fixture without the attr.
    assert!(!product_of::<Named>().allow_unknown_tags);
}

#[test]
fn allow_unknown_tags_shows_up_on_the_registry_entry() {
    let mut reg = TagRegistry::new();
    LenientType::register_into(&mut reg);
    let entry = reg.get("LenientType").expect("LenientType should register itself");
    assert!(entry_product(entry).allow_unknown_tags);
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

/// The public type's own tagged shape is inert (empty product) — the wire
/// shape comes from the wire DTO, which has its own non-empty `fields`.
#[test]
fn via_public_type_constants_are_empty() {
    let p = product_of::<Public>();
    assert!(p.fields.is_empty());
    assert!(p.reserved.is_empty());
    assert!(p.defaults.is_empty());
    // The wire DTO's fields, by contrast, are populated from its own #[tag(N)].
    assert_eq!(product_of::<WireDto>().fields, &[(0, "payload")]);
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
fn enum_variants_are_emitted_in_tag_order_with_variant_names() {
    // Source: Multi (#[tag(2)]), Nothing (#[tag(0)]), Single (#[tag(1)]).
    // After tag-ascending sort, the variant idents land in tag order.
    assert_eq!(
        variant_pairs(sum_of::<Choice>()),
        vec![(0, "Nothing"), (1, "Single"), (2, "Multi")],
    );
}

#[test]
fn enum_register_into_populates_registry_under_type_name() {
    let mut reg = TagRegistry::new();
    Choice::register_into(&mut reg);
    let entry = reg.get("Choice").expect("Choice should register itself");
    assert_eq!(variant_pairs(entry_sum(entry)), vec![(0, "Nothing"), (1, "Single"), (2, "Multi")],);
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
    assert_eq!(sum_of::<WithReservedVariants>().reserved, &[1, 2]);

    let mut reg = TagRegistry::new();
    WithReservedVariants::register_into(&mut reg);
    let entry = reg.get("WithReservedVariants").expect("should register itself");
    let s = entry_sum(entry);
    assert!(s.is_reserved(1));
    assert!(s.is_reserved(2));
    assert!(!s.is_reserved(0));
    assert!(!s.is_reserved(3));
}

/// Named-variant payloads pick up `#[tag(N)]` annotations on every field —
/// same rule as a top-level named struct. The variant-payload `Product`'s
/// `fields` slice lists `(field_tag, field_name)` pairs in tag-ascending
/// order.
#[test]
fn named_variant_payload_has_field_tags() {
    let s = sum_of::<Choice>();
    let multi = s.variant_for("Multi").expect("Multi variant exists");
    assert_eq!(multi.payload.fields, &[(0, "a"), (1, "b")]);
}

/// Unit variants and single-element tuple ("newtype-style") variants flow
/// through the same `parse_tuple_fields` helper as multi-element tuples,
/// using implicit positional tags when the user doesn't supply explicit
/// `#[tag(N)]` annotations on the payload field.
#[test]
fn variant_payload_shapes_pick_the_right_field_layout() {
    let s = sum_of::<WithInnerPayload>();

    let empty = s.variant_for("Empty").unwrap();
    assert!(empty.payload.fields.is_empty(), "unit variant has no fields");

    let one_inner = s.variant_for("OneInner").unwrap();
    assert_eq!(
        one_inner.payload.fields,
        &[(0, "0")],
        "single-element tuple variant gets implicit positional tag 0",
    );

    let pair = s.variant_for("Pair").unwrap();
    assert_eq!(
        pair.payload.fields,
        &[(0, "left"), (1, "right")],
        "named-variant fields keep their declared `#[tag(N)]`s",
    );
}

/// Tuple variants follow the all-or-nothing rule: explicit `#[tag(N)]` on
/// every field allows reordering and `default`, just like top-level tuple
/// structs. `Triple` declares tags out of source order to verify
/// tag-ascending sorting on the wire.
#[test]
fn explicit_tuple_variant_payload_sorts_and_carries_defaults() {
    let s = sum_of::<ExplicitTupleVariants>();
    let triple = s.variant_for("Triple").unwrap();
    assert_eq!(triple.payload.fields, &[(0, "1"), (1, "2"), (2, "0")]);
    assert_eq!(triple.payload.defaults, &[1]);

    let pair = s.variant_for("ImplicitPair").unwrap();
    assert_eq!(pair.payload.fields, &[(0, "0"), (1, "1")]);
    assert!(pair.payload.defaults.is_empty());
}

/// Inside a named variant payload, `#[tag(skip)]` and `PhantomData<_>`
/// behave the same as in a top-level named struct — neither shows up on
/// the wire and neither contributes to the type's bound chain.
#[test]
fn variant_payload_supports_skip_and_phantom() {
    let s = sum_of::<WithVariantFieldExtras<Opaque>>();
    let plain = s.variant_for("Plain").unwrap();
    assert_eq!(
        plain.payload.fields,
        &[(0, "required"), (1, "annotation")],
        "skipped and phantom-data fields don't appear in the variant payload",
    );
    assert_eq!(plain.payload.defaults, &[1]);
}

/// Variant payloads still don't carry their own `reserved` list or
/// `allow_unknown_tags` flag — there's no per-variant `#[tagged(...)]`
/// syntax for those yet. Pinning the empty defaults so a future change
/// to add per-variant attrs has to update this test.
#[test]
#[allow(clippy::const_is_empty)]
fn variant_payloads_have_no_reserved_or_allow_unknown_tags_yet() {
    for variant in sum_of::<Choice>().variants {
        assert!(variant.payload.reserved.is_empty(), "{} payload reserved", variant.name);
        assert!(!variant.payload.allow_unknown_tags, "{} payload allow_unknown_tags", variant.name);
    }
}

/// Default decode policy for any enum that doesn't opt in is strict on both
/// ends — encountering a reserved or unknown variant tag is an error.
#[test]
#[allow(clippy::assertions_on_constants)]
fn default_decode_policy_is_strict_for_plain_enums() {
    let s = sum_of::<Choice>();
    assert!(!s.default_on_reserved);
    assert!(!s.default_on_unknown);
    let s = sum_of::<WithReservedVariants>();
    assert!(!s.default_on_reserved, "reserved alone doesn't imply default-fallback");
    assert!(!s.default_on_unknown);
}

#[test]
fn default_on_reserved_flag_propagates_into_sum() {
    let s = sum_of::<BackwardsCompat>();
    assert!(s.default_on_reserved);
    assert!(!s.default_on_unknown);
    assert_eq!(s.reserved, &[2]);
}

#[test]
fn default_on_unknown_flag_propagates_into_sum() {
    let s = sum_of::<ForwardCompat>();
    assert!(!s.default_on_reserved);
    assert!(s.default_on_unknown);
}

#[test]
fn both_decode_policy_flags_can_combine() {
    let s = sum_of::<FullyLenient>();
    assert!(s.default_on_reserved);
    assert!(s.default_on_unknown);
    assert_eq!(s.reserved, &[7]);
}

/// The decode-policy flags also reach the registry entry — the wrapper
/// will read them off the `Sum` shape on the entry, not off the trait const.
#[test]
fn decode_policy_flags_show_up_on_the_registry_entry() {
    let mut reg = TagRegistry::new();
    FullyLenient::register_into(&mut reg);
    let entry = reg.get("FullyLenient").expect("FullyLenient should register itself");
    let s = entry_sum(entry);
    assert!(s.default_on_reserved);
    assert!(s.default_on_unknown);
}
