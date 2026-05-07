//! End-to-end smoke tests for the `MsgpackTagged` derive.
//!
//! Currently the derive is a stub: it produces a syntactically valid impl
//! with empty `TAGS` / `RESERVED` and a no-op `register_into`. These tests
//! check the scaffold compiles for each of the basic shapes — unit struct,
//! struct with named fields, enum with multiple kinds of variants. As real
//! expansion logic lands in subsequent steps, the tests in this file will
//! grow to assert the expected `TAGS` / registry-population behavior.

// Test fixtures only exist to feed the derive; unused fields/variants are
// expected.
#![allow(dead_code)]

use msgpack_tagged::{MsgpackTagged, TagRegistry};

#[derive(MsgpackTagged)]
struct Unit;

#[derive(MsgpackTagged)]
struct Tuple(u32, bool);

#[derive(MsgpackTagged)]
struct Named {
    a: u32,
    b: bool,
}

#[derive(MsgpackTagged)]
enum Choice {
    Nothing,
    Single(u32),
    Multi { a: u32, b: bool },
}

/// The mere fact that each `#[derive(MsgpackTagged)]` above expands without a
/// compile error is the test. This explicit instantiation makes failures
/// surface as a missing-bound error rather than dead-code warnings.
#[test]
fn derive_compiles_for_basic_shapes() {
    fn assert_impl<T: MsgpackTagged>() {}
    assert_impl::<Unit>();
    assert_impl::<Tuple>();
    assert_impl::<Named>();
    assert_impl::<Choice>();
}

/// Stub expansion contract: the derive emits empty `TAGS`/`RESERVED` and a
/// `register_into` that does nothing. These assertions will be tightened as
/// real expansion logic lands.
#[test]
fn stub_expansion_emits_empty_metadata() {
    assert!(<Unit as MsgpackTagged>::TAGS.is_empty());
    assert!(<Named as MsgpackTagged>::TAGS.is_empty());
    assert!(<Choice as MsgpackTagged>::TAGS.is_empty());

    let mut reg = TagRegistry::new();
    Unit::register_into(&mut reg);
    Named::register_into(&mut reg);
    Choice::register_into(&mut reg);
    assert!(reg.is_empty());
}
