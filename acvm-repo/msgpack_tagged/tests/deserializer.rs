//! Integration tests for the public decode path
//! ([`msgpack_tagged::msgpack_tagged_deserialize`]).
//!
//! Most tests round-trip a value through encode + decode and assert
//! equality; that catches both side-of-the-wrapper bugs in one shot.
//! One test (`tuple_struct_decodes_from_tag_ascending_wire_order`) hand-
//! builds a wire to lock in shape-specific decode behavior. Lives under
//! `tests/` rather than `#[cfg(test)]` inside `src/deserializer.rs` to
//! keep that source file readable.

#![allow(dead_code)]

use msgpack_tagged::{MsgpackTagged, msgpack_tagged_deserialize, msgpack_tagged_serialize};

/// Round-trip `value` through `msgpack_tagged_serialize` then
/// `msgpack_tagged_deserialize` and assert it survives unchanged. The
/// shared shape every interception test uses.
fn assert_roundtrip<T>(value: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + MsgpackTagged + PartialEq + std::fmt::Debug,
{
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded: T = msgpack_tagged_deserialize(&bytes).expect("deserialize succeeds");
    assert_eq!(decoded, value);
}

/// Sequence of primitives — basic round-trip through both
/// `serialize_seq` and `deserialize_seq`'s adapter.
#[test]
fn vec_of_primitives_roundtrips() {
    let value: Vec<u32> = vec![1, 2, 3, 4, 5];
    assert_roundtrip(value);
}

/// Empty sequence — exercises the zero-length array shape: the array
/// header is read, `next_element_seed` returns `Ok(None)` immediately,
/// no element bytes are read.
#[test]
fn empty_vec_roundtrips() {
    let value: Vec<u32> = vec![];
    assert_roundtrip(value);
}

/// Nested sequences — `Vec<Vec<u32>>` exercises `deserialize_seq`'s
/// adapter routing each element through `&mut *self.parent`, which
/// itself recurses into `deserialize_seq` for the inner Vec.
#[test]
fn nested_vec_roundtrips() {
    let value: Vec<Vec<u32>> = vec![vec![1, 2], vec![], vec![3, 4, 5]];
    assert_roundtrip(value);
}

/// `Vec<Option<T>>` exercises composition of `deserialize_seq` and
/// `deserialize_option`: each element calls our `deserialize_option`,
/// which peeks the marker and either visits None or restores +
/// recurses for Some. Mixed Some/None elements verify the per-element
/// state isolation.
#[test]
fn vec_of_options_roundtrips() {
    let value: Vec<Option<u32>> = vec![Some(1), None, Some(2), None, None, Some(3)];
    assert_roundtrip(value);
}

/// Fixed-length Rust tuple — same wire shape as a sequence (msgpack
/// array). `deserialize_tuple` shares the access adapter with
/// `deserialize_seq`, plus an eager length-mismatch check.
#[test]
fn tuple_roundtrips() {
    let value: (u32, bool, u8) = (7, true, 9);
    assert_roundtrip(value);
}

/// Tuple containing an option element exercises the recursion through
/// the wrapper — without it, `deserialize_option` would never see the
/// inner `Some(_)` value.
#[test]
fn tuple_with_option_roundtrips() {
    let value: (u32, Option<u32>, Option<u32>) = (1, Some(2), None);
    assert_roundtrip(value);
}

/// `None` round-trips: nil on the wire, `visit_none` on decode.
#[test]
fn option_none_roundtrips() {
    let value: Option<u32> = None;
    assert_roundtrip(value);
}

/// `Some(<primitive>)` round-trips. Our `deserialize_option` peeks the
/// marker, restores the reader's position, and routes the inner value
/// through `&mut *self`.
#[test]
fn option_some_with_primitive_roundtrips() {
    let value: Option<u32> = Some(42);
    assert_roundtrip(value);
}

/// `Some(Some(<primitive>))` exercises the recursive case — our
/// `deserialize_option` calls `visit_some(&mut *self)`, and the inner
/// `Option<u32>::deserialize` then calls our `deserialize_option`
/// again. Peek + restore must compose correctly.
#[test]
fn nested_option_roundtrips() {
    let value: Option<Option<u32>> = Some(Some(7));
    assert_roundtrip(value);
}

/// Newtype struct around a primitive — wire bytes are the inner
/// value's bytes alone (no wrapping). Round-trip exercises
/// `deserialize_newtype_struct` calling `visitor.visit_newtype_struct(&mut *self)`,
/// which dispatches to our `deserialize_u32` for the inner.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct Witness(u32);

#[test]
fn newtype_struct_with_primitive_roundtrips() {
    assert_roundtrip(Witness(42));
}

/// Generic newtype around a tagged inner type. The inner deserializer
/// invocation must go through our wrapper (not rmp_serde's inner) so
/// the inner type's `deserialize_seq` / etc. ends up at our
/// interception once those methods land.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct Wrapper<T>(T);

#[test]
fn newtype_struct_with_tagged_inner_roundtrips() {
    assert_roundtrip(Wrapper(Witness(7)));
}

/// Newtype struct carrying an Option exercises both interception
/// paths: outer `deserialize_newtype_struct` recurses through us,
/// then the inner `deserialize_option` does its peek/restore dance.
#[test]
fn newtype_struct_with_option_inner_roundtrips() {
    assert_roundtrip(Wrapper(Some(99u32)));
    assert_roundtrip(Wrapper::<Option<u32>>(None));
}

/// Free-form map (`BTreeMap`) with primitive keys and values —
/// basic round-trip through both `serialize_map` and our
/// `deserialize_map` adapter.
#[test]
fn btree_map_roundtrips() {
    use std::collections::BTreeMap;
    let mut value: BTreeMap<u32, u32> = BTreeMap::new();
    value.insert(1, 100);
    value.insert(2, 200);
    value.insert(3, 300);
    assert_roundtrip(value);
}

/// Empty map — exercises the zero-length map shape.
#[test]
fn empty_btree_map_roundtrips() {
    use std::collections::BTreeMap;
    let value: BTreeMap<u32, u32> = BTreeMap::new();
    assert_roundtrip(value);
}

/// Map values that themselves need interception — verifies
/// `next_value_seed` routes through `&mut *self.parent`.
#[test]
fn btree_map_with_option_values_roundtrips() {
    use std::collections::BTreeMap;
    let mut value: BTreeMap<u8, Option<u32>> = BTreeMap::new();
    value.insert(0, Some(7));
    value.insert(1, None);
    value.insert(2, Some(11));
    assert_roundtrip(value);
}

/// Map values containing a sequence — composes map + seq
/// interception. Each value is a Vec, decoded via our
/// `deserialize_seq`.
#[test]
fn btree_map_with_vec_values_roundtrips() {
    use std::collections::BTreeMap;
    let mut value: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    value.insert(0, vec![1, 2, 3]);
    value.insert(1, vec![]);
    value.insert(2, vec![4, 5]);
    assert_roundtrip(value);
}

/// Named struct round-trip — exercises the load-bearing path:
/// `serialize_struct` writes an int-keyed map; `deserialize_struct`
/// reads each `(int_tag, value)` pair, looks up the tag in the
/// registered `Product`, and yields the field name to the visitor.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct Pair {
    #[tag(2)]
    first: u32,
    #[tag(5)]
    second: bool,
}

#[test]
fn named_struct_roundtrips() {
    assert_roundtrip(Pair { first: 7, second: true });
}

/// Reordered tags: source-declaration order doesn't match
/// tag-ascending order. Since serde's visitor matches by *name*, this
/// works regardless of which order entries appear on the wire.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct OutOfOrder {
    #[tag(2)]
    c: u32,
    #[tag(0)]
    a: u32,
    #[tag(1)]
    b: u32,
}

#[test]
fn struct_with_out_of_order_tags_roundtrips() {
    assert_roundtrip(OutOfOrder { c: 30, a: 10, b: 20 });
}

/// Nested tagged struct — the outer struct's `deserialize_struct`
/// recurses through `&mut *self.parent` for each value, hitting our
/// `deserialize_struct` again for the inner field. Tag translation
/// must compose correctly.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct Outer {
    #[tag(0)]
    nested: Pair,
    #[tag(1)]
    flag: u8,
}

#[test]
fn nested_tagged_struct_roundtrips() {
    assert_roundtrip(Outer { nested: Pair { first: 99, second: false }, flag: 7 });
}

/// Struct field that's a `Vec<Tagged>` — composes struct + seq
/// interception. Each element of the Vec is itself a tagged struct.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct WithVecField {
    #[tag(0)]
    items: Vec<Pair>,
}

#[test]
fn struct_with_vec_of_tagged_field_roundtrips() {
    let value = WithVecField {
        items: vec![Pair { first: 1, second: true }, Pair { first: 2, second: false }],
    };
    assert_roundtrip(value);
}

/// `#[tag(N, default)]` paired with `#[serde(default)]` — wire
/// tolerance is delegated to serde-derive's standard `default`
/// machinery. Encode the full value (so the wire has all tags), then
/// round-trip — verifies the basic shape compiles and works without
/// the user needing to construct partial wire bytes.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug, Default)]
struct WithDefaults {
    #[tag(0)]
    required: u32,
    #[tag(1, default)]
    #[serde(default)]
    annotation: Vec<u8>,
}

#[test]
fn struct_with_defaults_roundtrips_when_all_present() {
    assert_roundtrip(WithDefaults { required: 7, annotation: vec![1, 2, 3] });
}

/// Multi-element tuple struct with implicit positional tags — the
/// load-bearing test for `deserialize_tuple_struct`. Wire is an
/// int-keyed map; for each source position the deserializer looks up
/// `Product.tag_for("N")` and finds the matching entry by tag.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct PositionalTriple(u32, bool, u8);

#[test]
fn tuple_struct_roundtrips() {
    assert_roundtrip(PositionalTriple(7, true, 9));
}

/// Tuple struct with explicit `#[tag(N)]` reordering tags relative to
/// source position — round-trips because the serializer writes in
/// source-position order and the deserializer reads positionally,
/// discarding the wire tag.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct ReorderedTriple(#[tag(2)] u32, #[tag(0)] bool, #[tag(1)] u8);

#[test]
fn tuple_struct_with_explicit_tags_roundtrips() {
    assert_roundtrip(ReorderedTriple(7, true, 9));
}

/// Tuple struct holding a tagged inner element — verifies recursion
/// through the wrapper for the value side.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
struct TupleWithNested(Pair, u8);

#[test]
fn tuple_struct_with_nested_tagged_element_roundtrips() {
    assert_roundtrip(TupleWithNested(Pair { first: 1, second: false }, 42));
}

/// Manually construct a tuple-struct wire in *tag-ascending* order
/// (the design-doc-intended order, which differs from the current
/// serializer's source-declaration order) and decode it. Verifies
/// that the deserializer reconstructs by tag rather than wire
/// position — i.e. it would still work if the serializer is later
/// fixed to emit tag-ascending. For `ReorderedTriple` (tags 2, 0, 1
/// at source positions 0, 1, 2), tag-ascending wire is
/// `{0: bool, 1: u8, 2: u32}`.
#[test]
fn tuple_struct_decodes_from_tag_ascending_wire_order() {
    let mut bytes: Vec<u8> = Vec::new();
    rmp::encode::write_map_len(&mut bytes, 3).expect("map header");
    // tag 0 → bool true (the source-position-1 field)
    rmp::encode::write_pfix(&mut bytes, 0).expect("tag 0");
    rmp::encode::write_bool(&mut bytes, true).expect("bool");
    // tag 1 → u8 9 (source-position-2 field)
    rmp::encode::write_pfix(&mut bytes, 1).expect("tag 1");
    rmp::encode::write_pfix(&mut bytes, 9).expect("u8 9");
    // tag 2 → u32 7 (source-position-0 field)
    rmp::encode::write_pfix(&mut bytes, 2).expect("tag 2");
    rmp::encode::write_pfix(&mut bytes, 7).expect("u32 7");

    let decoded: ReorderedTriple = msgpack_tagged_deserialize(&bytes).expect("decode");
    assert_eq!(decoded, ReorderedTriple(7, true, 9));
}
