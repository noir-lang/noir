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
