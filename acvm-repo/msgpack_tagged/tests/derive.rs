//! End-to-end tests for the `MsgpackTagged` derive on structs and enums.
//!
//! Unit structs still fall through to the stub expansion (empty `Tagged::Product`,
//! no-op `register_into`); their tests here just verify the derive produces a
//! valid impl. As the macro learns to handle each shape, the corresponding
//! tests in this file get tightened.

// Test fixtures only exist to feed the derive; unused fields are expected.
#![allow(dead_code)]

use msgpack_tagged::{Entry, MsgpackTagged, Product, Sum, TagRegistry, VariantKind};

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

/// Variant-level `#[tagged(reserved(...))]` and `#[tagged(allow_unknown_tags)]`
/// configure each variant's payload independently. `Strict` keeps the strict
/// default; `Lenient` opts in to skipping unknown payload field tags and
/// retires payload tag 5; `Brief` retires payload tag 9 without lenience.
/// The variant-level `reserved` lists govern *payload field* tags only —
/// they don't interact with the enclosing enum's variant-tag space.
#[derive(MsgpackTagged)]
enum VariantPayloadConfigs {
    #[tag(0)]
    Strict {
        #[tag(0)]
        a: u32,
    },
    #[tag(1)]
    #[tagged(reserved(5), allow_unknown_tags)]
    Lenient {
        #[tag(0)]
        b: u32,
        #[tag(1)]
        c: bool,
    },
    #[tag(2)]
    #[tagged(reserved(9))]
    Brief(#[tag(0)] u32, #[tag(2)] bool),
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

/// Self-recursive enum: `Branch` carries `Vec<Tree>` (self-typed). The macro
/// detects the self-reference and skips emitting the `Vec<Tree>: MsgpackTagged`
/// where-clause bound that would otherwise trigger a co-inductive cycle in
/// the trait solver. The recursion call inside `register_into` is still
/// emitted; Rust's call-site resolution accepts it co-inductively against
/// the impl being defined.
#[derive(MsgpackTagged)]
enum Tree {
    #[tag(0)]
    Leaf(u32),
    #[tag(1)]
    Branch(Vec<Tree>),
}

/// Self-recursive struct: `parent` is `Option<Box<Cons>>`, also self-typed.
/// Same handling as the enum case — bound dropped, recursion call retained.
#[derive(MsgpackTagged)]
struct Cons {
    #[tag(0)]
    value: u32,
    #[tag(1)]
    parent: Option<Box<Cons>>,
}

/// Self-recursive enum where the recursive variant *also* contains a
/// sibling type. The self-filter drops the `Vec<(Inner, NestedTree)>: MsgpackTagged`
/// bound (because it contains `Self`), but the recursion call still needs
/// `Inner: MsgpackTagged` to resolve at the call site. `extra_bound`
/// restores it; without it, the impl would fail to compile with a clear
/// "Inner: MsgpackTagged is not satisfied" error.
#[derive(MsgpackTagged)]
#[tagged(extra_bound = "Inner: msgpack_tagged::MsgpackTagged")]
enum NestedTree {
    #[tag(0)]
    Leaf(Inner),
    #[tag(1)]
    Branch(Vec<(Inner, NestedTree)>),
}

/// Multiple `extra_bound` attributes accumulate — each one's predicates are
/// appended to the impl's where clause. The two extra bounds here are
/// equivalent to a single `extra_bound = "Inner: ..., u32: ..."` but split
/// across attributes for clarity.
#[derive(MsgpackTagged)]
#[tagged(extra_bound = "Inner: msgpack_tagged::MsgpackTagged")]
#[tagged(extra_bound = "u32: msgpack_tagged::MsgpackTagged")]
enum NestedTreeMulti {
    #[tag(0)]
    Leaf(Inner),
    #[tag(1)]
    Branch(Vec<(Inner, u32, NestedTreeMulti)>),
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

/// Field-level `#[serde(rename = "...")]` overrides the wire-name in
/// `Product.fields` for that field — load-bearing for shadow-DTOs whose
/// wire DTO renames individual fields. Field 0 is renamed `index` → `i`,
/// field 1 keeps its Rust ident.
#[derive(MsgpackTagged, serde::Serialize)]
struct WireWithRenamedFields {
    #[serde(rename = "i")]
    #[tag(0)]
    index: u32,
    #[tag(1)]
    value: bool,
}

/// `Opaque` doesn't implement `serde::Serialize`, but `#[serde(skip)]`
/// drops the field before serde inspects it — so a type with a skipped
/// `Opaque` field still derives `Serialize` cleanly. Used by the fixtures
/// below to keep `#[serde(skip)]` recognized as an attribute.
impl serde::Serialize for Opaque {
    fn serialize<S: serde::Serializer>(&self, _s: S) -> Result<S::Ok, S::Error> {
        unreachable!("Opaque is `#[serde(skip)]`-ed in every fixture that touches it")
    }
}

/// `#[serde(skip)]` is recognized as an alias for `#[tag(skip)]` — the
/// field is dropped from the wire and contributes no bound. `Opaque`
/// doesn't implement `MsgpackTagged`; the impl still compiles because
/// the field is skipped.
#[derive(MsgpackTagged, serde::Serialize)]
struct WithSerdeSkip {
    #[tag(0)]
    visible: u32,
    #[serde(skip)]
    hidden: Opaque,
}

/// `#[serde(skip)]` works inside a named variant payload too — the
/// payload field is dropped, no `MsgpackTagged` bound on its type.
#[derive(MsgpackTagged, serde::Serialize)]
enum WithSerdeSkipInVariant {
    #[tag(0)]
    Plain {
        #[tag(0)]
        keep: u32,
        #[serde(skip)]
        drop: Opaque,
    },
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
    assert_impl::<VariantPayloadConfigs>();
    assert_impl::<GenericChoice<u32>>();
    assert_impl::<GenericChoice<Inner>>();
    assert_impl::<WithReservedVariants>();
    assert_impl::<BackwardsCompat>();
    assert_impl::<ForwardCompat>();
    assert_impl::<FullyLenient>();
    // Self-recursive types: the macro's self-filter drops the
    // `Vec<Self>: MsgpackTagged` / `Option<Box<Self>>: MsgpackTagged` bound
    // that would otherwise trigger a co-inductive trait-solver cycle.
    assert_impl::<Tree>();
    assert_impl::<Cons>();
    // Self-recursive with sibling: `extra_bound` restores the sibling's
    // `MsgpackTagged` bound that the self-filter would otherwise have
    // implied via the (now-dropped) `Vec<(Inner, Self)>` bound.
    assert_impl::<NestedTree>();
    assert_impl::<NestedTreeMulti>();
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
    assert_impl::<WireWithRenamedFields>();
    assert_impl::<WithSerdeSkip>();
    assert_impl::<WithSerdeSkipInVariant>();
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

/// Each variant shape lands on the right `VariantKind` discriminator with
/// the matching payload — unit and newtype variants both have empty
/// `payload.fields`, distinguished only by the `kind`. Struct (named-field)
/// variants populate `payload.fields` from `#[tag(N)]` annotations.
#[test]
fn variant_payload_shapes_pick_the_right_field_layout() {
    let s = sum_of::<WithInnerPayload>();

    let empty = s.variant_for("Empty").unwrap();
    assert_eq!(empty.kind, VariantKind::Unit);
    assert!(empty.payload.fields.is_empty(), "unit variant has no fields");

    let one_inner = s.variant_for("OneInner").unwrap();
    assert_eq!(
        one_inner.kind,
        VariantKind::Newtype,
        "single-element tuple variant is a newtype variant",
    );
    assert!(
        one_inner.payload.fields.is_empty(),
        "newtype variants pass through to the inner type — no field tag map of their own",
    );

    let pair = s.variant_for("Pair").unwrap();
    assert_eq!(pair.kind, VariantKind::Struct);
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
    assert_eq!(triple.kind, VariantKind::Tuple);
    assert_eq!(triple.payload.fields, &[(0, "1"), (1, "2"), (2, "0")]);
    assert_eq!(triple.payload.defaults, &[1]);

    let pair = s.variant_for("ImplicitPair").unwrap();
    assert_eq!(pair.kind, VariantKind::Tuple);
    assert_eq!(pair.payload.fields, &[(0, "0"), (1, "1")]);
    assert!(pair.payload.defaults.is_empty());
}

/// Each variant ends up with the right `VariantKind` discriminator: unit
/// variants get `Unit`, single-element tuple variants get `Newtype`,
/// multi-element tuple variants get `Tuple`, and named-field variants get
/// `Struct`. `Choice` is the canonical mixed-shape fixture covering three of
/// these in a single enum.
#[test]
fn variant_kinds_match_each_variant_shape() {
    let s = sum_of::<Choice>();
    assert_eq!(s.variant_for("Nothing").unwrap().kind, VariantKind::Unit);
    assert_eq!(s.variant_for("Single").unwrap().kind, VariantKind::Newtype);
    assert_eq!(s.variant_for("Multi").unwrap().kind, VariantKind::Struct);

    let s = sum_of::<ExplicitTupleVariants>();
    assert_eq!(
        s.variant_for("Triple").unwrap().kind,
        VariantKind::Tuple,
        "three-element tuple variant is a Tuple, not a Newtype",
    );
}

/// Newtype variants carry no payload field tags of their own — the inner
/// type's bytes go directly under the variant tag on the wire. `payload` is
/// the empty `Product` (same shape a unit variant gets); the only thing
/// distinguishing them at the metadata level is `kind`. This is the
/// invariant the wrapper relies on to skip field-table lookup for newtype
/// variants and pass through the inner value unwrapped.
#[test]
#[allow(clippy::const_is_empty)]
fn newtype_variant_metadata_is_empty_payload_with_newtype_kind() {
    let s = sum_of::<WithInnerPayload>();
    let one_inner = s.variant_for("OneInner").unwrap();
    assert_eq!(one_inner.kind, VariantKind::Newtype);
    assert!(one_inner.payload.fields.is_empty());
    assert!(one_inner.payload.defaults.is_empty());
    assert!(one_inner.payload.reserved.is_empty());
    assert!(!one_inner.payload.allow_unknown_tags);
}

/// Unit variants carry no payload at all — same empty `Product` as newtype
/// variants, distinguished only by `kind == Unit`.
#[test]
#[allow(clippy::const_is_empty)]
fn unit_variant_metadata_is_empty_payload_with_unit_kind() {
    let s = sum_of::<WithInnerPayload>();
    let empty = s.variant_for("Empty").unwrap();
    assert_eq!(empty.kind, VariantKind::Unit);
    assert!(empty.payload.fields.is_empty());
    assert!(empty.payload.defaults.is_empty());
    assert!(empty.payload.reserved.is_empty());
    assert!(!empty.payload.allow_unknown_tags);
}

/// Newtype variants reach their inner type via the impl's where clause and
/// `register_into` recursion, even though they don't appear in `payload`.
/// `Choice::Single(u32)` is a newtype variant of `u32` — the registry walk
/// for `Choice` therefore must transitively touch `u32`'s (no-op) `register_into`
/// without errors. We verify the recursion compiles and runs by walking
/// `WithInnerPayload`, whose `OneInner(Inner)` newtype variant should reach
/// the registry-recording `Inner` type.
#[test]
fn newtype_variant_recursion_reaches_inner_type() {
    let mut reg = TagRegistry::new();
    WithInnerPayload::register_into(&mut reg);
    assert!(
        reg.get("Inner").is_some(),
        "newtype variant's inner type is reached via the bound chain even though \
         payload.fields is empty",
    );
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

/// Variants without a `#[tagged(...)]` annotation default to a strict
/// payload — empty `reserved` and `allow_unknown_tags = false`.
#[test]
#[allow(clippy::const_is_empty)]
fn variant_payload_defaults_are_strict_when_no_tagged_attr() {
    for variant in sum_of::<Choice>().variants {
        assert!(variant.payload.reserved.is_empty(), "{} payload reserved", variant.name);
        assert!(!variant.payload.allow_unknown_tags, "{} payload allow_unknown_tags", variant.name);
    }
}

/// Per-variant `#[tagged(reserved(...))]` and `#[tagged(allow_unknown_tags)]`
/// land on the variant's payload `Product`, not on the enum's `Sum`. Each
/// variant's policy is independent of the others.
#[test]
fn variant_level_tagged_attrs_propagate_per_variant() {
    let s = sum_of::<VariantPayloadConfigs>();

    let strict = s.variant_for("Strict").expect("Strict variant exists");
    assert!(strict.payload.reserved.is_empty());
    assert!(!strict.payload.allow_unknown_tags);

    let lenient = s.variant_for("Lenient").expect("Lenient variant exists");
    assert_eq!(lenient.payload.reserved, &[5]);
    assert!(lenient.payload.allow_unknown_tags);

    let brief = s.variant_for("Brief").expect("Brief variant exists");
    assert_eq!(brief.payload.reserved, &[9]);
    assert!(!brief.payload.allow_unknown_tags);

    // Variant-level reserved doesn't bleed into the enum-level reserved
    // variant-tag list — that space is governed by the type-level
    // `#[tagged(reserved(...))]`.
    assert!(s.reserved.is_empty(), "enum-level reserved is independent");
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

/// Field-level `#[serde(rename = "X")]` rewrites the wire name in
/// `Product.fields` so the wrapper's `tag_for("X")` matches what serde's
/// `serialize_field("X", ...)` will pass at runtime.
#[test]
fn serde_field_rename_drives_wire_field_name() {
    let p = product_of::<WireWithRenamedFields>();
    assert_eq!(
        p.fields,
        &[(0, "i"), (1, "value")],
        "renamed field uses the rename target; un-renamed field keeps its Rust ident",
    );
    assert_eq!(p.tag_for("i"), Some(0), "wrapper lookup for the renamed name should hit");
    assert_eq!(p.tag_for("index"), None, "Rust ident must NOT be used after a rename");
}

/// `#[serde(skip)]` is honored as an alias for `#[tag(skip)]` — the field
/// is absent from `Product.fields` and the type compiles even when the
/// skipped field's type doesn't implement `MsgpackTagged`.
#[test]
fn serde_skip_is_an_alias_for_tag_skip() {
    let p = product_of::<WithSerdeSkip>();
    assert_eq!(p.fields, &[(0, "visible")], "`#[serde(skip)]` field is absent from fields");
}

#[test]
fn serde_skip_works_inside_variant_payload() {
    let s = sum_of::<WithSerdeSkipInVariant>();
    let plain = s.variant_for("Plain").unwrap();
    assert_eq!(plain.payload.fields, &[(0, "keep")]);
}

/// Self-recursive enum: the derive emits `Vec<Tree>` as the `Branch` payload
/// field and the `register_into` body still recurses into `<Vec<Tree>>` —
/// only the where-clause bound was dropped to break the cycle. At runtime,
/// `try_insert` short-circuits the inner `Self::register_into` call.
#[test]
fn self_recursive_enum_registers_itself_once() {
    let mut reg = TagRegistry::new();
    Tree::register_into(&mut reg);
    Tree::register_into(&mut reg);
    let entry = reg.get("Tree").expect("Tree should register itself");
    assert_eq!(reg.len(), 1, "self-recursion is bounded by `try_insert` idempotency");
    let pairs: Vec<_> = entry_sum(entry).variants.iter().map(|v| (v.tag, v.name)).collect();
    assert_eq!(pairs, vec![(0, "Leaf"), (1, "Branch")]);
}

/// Self-recursive struct: same idea, struct shape — `parent` field's type
/// is `Option<Box<Cons>>`, so the bound chain hits `Cons: MsgpackTagged`
/// (the impl being defined). The self-filter drops the bound; the impl
/// compiles and registers correctly.
#[test]
fn self_recursive_struct_registers_itself_once() {
    let mut reg = TagRegistry::new();
    Cons::register_into(&mut reg);
    let entry = reg.get("Cons").expect("Cons should register itself");
    assert_eq!(reg.len(), 1);
    assert_eq!(entry_product(entry).fields, &[(0, "value"), (1, "parent")]);
}

/// `extra_bound` reaches the sibling type. After `NestedTree::register_into`
/// runs, both `NestedTree` and `Inner` are in the registry — `Inner` is
/// reached via the `Vec<(Inner, NestedTree)>` recursion call (which compiles
/// because the `extra_bound` keeps `Inner: MsgpackTagged` in the where
/// clause).
#[test]
fn extra_bound_restores_sibling_type_in_self_recursive_payload() {
    let mut reg = TagRegistry::new();
    NestedTree::register_into(&mut reg);
    assert!(reg.get("NestedTree").is_some(), "self-recursive type registers itself");
    assert!(
        reg.get("Inner").is_some(),
        "sibling type reached via Vec<(Inner, NestedTree)> recursion",
    );
}
