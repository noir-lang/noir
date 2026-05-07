//! End-to-end tests for the `MsgpackTagged` derive on structs.
//!
//! Tuple structs and enums still fall through to the stub expansion (empty
//! `TAGS`, no-op `register_into`); their tests here just verify the derive
//! produces a valid impl. As the macro learns to handle each shape, the
//! corresponding tests in this file get tightened.

// Test fixtures only exist to feed the derive; unused fields are expected.
#![allow(dead_code)]

use msgpack_tagged::{MsgpackTagged, TagRegistry};

#[derive(MsgpackTagged)]
struct Unit;

#[derive(MsgpackTagged)]
struct Tuple(u32, bool);

#[derive(MsgpackTagged)]
struct Named {
    #[tag(0)]
    a: u32,
    #[tag(1)]
    b: bool,
}

#[derive(MsgpackTagged)]
enum Choice {
    Nothing,
    Single(u32),
    Multi { a: u32, b: bool },
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

#[test]
fn derive_compiles_for_basic_shapes() {
    fn assert_impl<T: MsgpackTagged>() {}
    assert_impl::<Unit>();
    assert_impl::<Tuple>();
    assert_impl::<Named>();
    assert_impl::<Choice>();
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
}

#[test]
fn unit_struct_has_empty_tags() {
    assert!(<Unit as MsgpackTagged>::TAGS.is_empty());
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
