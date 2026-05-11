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

/// `#[serde(default)]` field — wire tolerance is delegated to
/// serde-derive's standard `default` machinery (see
/// `v1_to_v2_add_field_with_default` for the missing-tag case). Encode
/// the full value (so the wire has all tags), then round-trip —
/// verifies the basic shape compiles and round-trips.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug, Default)]
struct WithDefaults {
    #[tag(0)]
    required: u32,
    #[tag(1)]
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

// ============================================================================
// Enum round-trip tests — one per `VariantKind` (unit / newtype / tuple /
// struct), plus a nested-tagged-payload case. Mirrors the
// `serializer::unit_variant_encodes_as_variant_tag_with_nil_payload`-style
// shape tests on the encode side, but here we just round-trip and assert
// equality — the bytes-shape locks are the serializer's job.
// ============================================================================

/// Mixed-shape enum exercising every `VariantKind`: same shape as the
/// serializer's `Mixed` fixture, plus the matching `Deserialize` derive.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
enum Mixed {
    #[tag(1)]
    Empty,
    #[tag(2)]
    Wrap(u32),
    #[tag(3)]
    Pair(u32, bool),
    #[tag(4)]
    Named {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    },
}

#[test]
fn unit_variant_roundtrips() {
    assert_roundtrip(Mixed::Empty);
}

#[test]
fn newtype_variant_roundtrips() {
    assert_roundtrip(Mixed::Wrap(42));
}

#[test]
fn tuple_variant_roundtrips() {
    assert_roundtrip(Mixed::Pair(7, true));
}

#[test]
fn struct_variant_roundtrips() {
    assert_roundtrip(Mixed::Named { a: 99, b: false });
}

/// Newtype variant carrying a tagged inner type — verifies the payload
/// recurses through the wrapper instead of falling back to rmp_serde for
/// the inner struct.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
enum NewtypeWithTagged {
    #[tag(7)]
    Wrap(Pair),
}

#[test]
fn newtype_variant_with_tagged_inner_roundtrips() {
    assert_roundtrip(NewtypeWithTagged::Wrap(Pair { first: 11, second: true }));
}

/// Tuple variant whose payload contains a tagged inner type — verifies
/// each payload position routes through the wrapper.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
enum TupleVariantWithNested {
    #[tag(0)]
    Carry(Pair, u32),
}

#[test]
fn tuple_variant_with_nested_tagged_element_roundtrips() {
    assert_roundtrip(TupleVariantWithNested::Carry(Pair { first: 1, second: false }, 5));
}

/// Struct variant whose payload contains a tagged inner type — same
/// recursion check from the struct-variant side.
#[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
enum StructVariantWithNested {
    #[tag(0)]
    Carry {
        #[tag(0)]
        inner: Pair,
        #[tag(1)]
        count: u32,
    },
}

#[test]
fn struct_variant_with_nested_tagged_element_roundtrips() {
    assert_roundtrip(StructVariantWithNested::Carry {
        inner: Pair { first: 2, second: true },
        count: 9,
    });
}

// ============================================================================
// Schema-evolution / cross-version tests for named structs.
//
// Each scenario lives in its own module so the V1/V2 fixtures can share
// the `Foo` family name (and `#[serde(rename = "Foo")]` to register under
// a stable key). Naming convention:
//   * V1 = the older snapshot
//   * V2 = the newer snapshot
//   * V1 → V2 = backward compat (write with old, read with new format)
//   * V2 → V1 = forward compat (with with new, read with old format)
// Tests use `#[should_panic(expected = "...")]` to capture *current*
// behavior when the design-doc-intended outcome is success but the
// implementation hasn't caught up yet — the panic substring locks in the
// failure mode that needs to flip once the relevant TODO is resolved.
// ============================================================================

/// V1 → V2: V2 retires a field by adding its tag to `reserved(...)` and
/// dropping the field declaration. V1 emits both fields on the wire; V2
/// silently skips the retired tag and decodes the rest. Mirror of the
/// enum `on_reserved` behavior but for products — the `reserved(...)`
/// list itself is the opt-in (no extra flag needed), since the only
/// retire-and-decode interpretation is "skip past the retired entry."
mod v1_to_v2_remove_field_with_reserved {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(reserved(1))]
    struct FooV2 {
        #[tag(0)]
        a: u32,
    }

    #[test]
    fn skip_retired_tag_on_decode() {
        let v1 = FooV1 { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2 { a: 7 });
    }
}

/// V1 → V2: V2 adds a new field marked `#[serde(default)]`. V1's wire
/// doesn't carry that tag; V2 fills the field from `Default` thanks to
/// serde-derive's standard `default` handling — no extra signaling from
/// our macro is required (the macro is purely about wire identity).
mod v1_to_v2_add_field_with_default {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        a: u32,
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug, Default)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        #[serde(default)]
        b: Vec<u8>,
    }

    #[test]
    fn missing_tag_uses_default() {
        let v1 = FooV1 { a: 7 };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2 { a: 7, b: Vec::default() });
    }
}

/// V1 → V2: V2 adds a new field with *no* default. V1's wire is missing
/// that tag; V2 decoding errors with `missing field …`.
mod v1_to_v2_add_new_required_field {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        a: u32,
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    }

    #[test]
    fn missing_required_tag_errors() {
        let v1 = FooV1 { a: 7 };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let err = msgpack_tagged_deserialize::<FooV2>(&bytes).expect_err("decode should fail");
        // serde-derive emits `missing field` for the absent required field;
        // accept any error mentioning the missing field name to stay
        // robust to wording changes.
        let msg = err.to_string();
        assert!(msg.contains("missing field") || msg.contains("`b`"), "got: {msg}");
    }
}

/// V1 → V2: V2 has the same fields and tags as V1, just declared in a
/// different source order. Since the wire-tag↔field-name mapping comes
/// from the registry (not source position), round-tripping just works.
mod v1_to_v2_reorder_fields {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(1)]
        b: bool,
        #[tag(0)]
        a: u32,
    }

    #[test]
    fn reorder_only_roundtrips() {
        let v1 = FooV1 { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2 { a: 7, b: true });
    }
}

/// V2 → V1: V2 has dropped a field that V1 still requires. V2's wire is
/// missing the field; V1 decoding errors with `missing field …`.
mod v2_to_v1_required_field_missing {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(0)]
        a: u32,
    }

    #[test]
    fn v1_required_field_missing_errors() {
        let v2 = FooV2 { a: 7 };
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let err = msgpack_tagged_deserialize::<FooV1>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("missing field") || msg.contains("`b`"), "got: {msg}");
    }
}

/// V2 → V1: V2 adds a new field that V1 doesn't know about, and V1 opts
/// into `allow_unknown_tags`. V1 silently skips the new entry.
mod v2_to_v1_extra_field_with_allow_unknown {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(allow_unknown_tags)]
    struct FooV1 {
        #[tag(0)]
        a: u32,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    }

    #[test]
    fn unknown_tag_skipped_when_allowed() {
        let v2 = FooV2 { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let v1: FooV1 = msgpack_tagged_deserialize(&bytes).expect("decode V1");
        assert_eq!(v1, FooV1 { a: 7 });
    }
}

/// V2 → V1: V2 adds a new field; V1 does *not* opt into `allow_unknown_tags`.
/// V1 errors on the unknown tag.
mod v2_to_v1_extra_field_without_allow_unknown {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        a: u32,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    }

    #[test]
    fn unknown_tag_errors_when_not_allowed() {
        let v2 = FooV2 { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let err = msgpack_tagged_deserialize::<FooV1>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("unknown wire tag"), "got: {msg}");
    }
}

/// V1 → V2: V2 renames a field's Rust ident while keeping the same tag.
/// Wire encoding never carries field names — only tags — so a rename is
/// invisible on the wire and round-trips freely. Useful when a developer
/// realizes the original name was misleading and wants to fix it without
/// breaking the wire format.
mod v1_to_v2_rename_field {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        field_a: u32,
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(0)]
        renamed_a: u32,
    }

    #[test]
    fn rename_keeping_same_tag_roundtrips() {
        let v1 = FooV1 { field_a: 7 };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2 { renamed_a: 7 });
    }
}

/// V2 → V1: same field set as V1, just reordered in source. Round-trips
/// because the wire carries tags, not field names — same as case
/// `v1_to_v2_reorder_fields` from the other direction.
mod v2_to_v1_reorder_fields {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV1 {
        #[tag(0)]
        a: u32,
        #[tag(1)]
        b: bool,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV2 {
        #[tag(1)]
        b: bool,
        #[tag(0)]
        a: u32,
    }

    #[test]
    fn reorder_only_roundtrips() {
        let v2 = FooV2 { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let v1: FooV1 = msgpack_tagged_deserialize(&bytes).expect("decode V1");
        assert_eq!(v1, FooV1 { a: 7, b: true });
    }
}

// ============================================================================
// Schema-evolution / cross-version tests for tuple structs.
//
// Tuple structs decode positionally — the visitor pulls fields by index, not
// by name — but the wire is still a tag-keyed map. The cases below verify
// that `deserialize_tuple_struct` tolerates wire-length drift the same way
// the named-struct path does: short wires fall through to serde-derive's
// `#[serde(default)]` machinery, long wires only round-trip when the type
// opts into `#[tagged(allow_unknown_tags)]`.
// ============================================================================

/// V1 → V2: V2 appends a new trailing element with `#[serde(default)]`. V1's
/// wire only carries the first position; V2's decode yields `Ok(None)` for
/// the missing position and serde-derive substitutes the default. Mirror of
/// the named-struct `v1_to_v2_add_field_with_default` case.
mod v1_to_v2_tuple_add_trailing_default {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV1(#[tag(0)] u32, #[tag(1)] bool);

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV2(
        #[tag(0)] u32,
        #[tag(1)] bool,
        #[tag(2)]
        #[serde(default)]
        Vec<u8>,
    );

    #[test]
    fn missing_trailing_position_uses_default() {
        let v1 = FooV1(7, true);
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2(7, true, Vec::default()));
    }
}

/// V2 → V1: V2 appends a new tuple position that V1 doesn't know about, and
/// V1 opts into `#[tagged(allow_unknown_tags)]`. The extra wire entry is
/// buffered but never queried by the visitor and gets discarded.
mod v2_to_v1_tuple_extra_with_allow_unknown {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(allow_unknown_tags)]
    struct FooV1(#[tag(0)] u32, #[tag(1)] bool);

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV2(#[tag(0)] u32, #[tag(1)] bool, #[tag(2)] u8);

    #[test]
    fn extra_trailing_position_skipped_when_allowed() {
        let v2 = FooV2(7, true, 42);
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let v1: FooV1 = msgpack_tagged_deserialize(&bytes).expect("decode V1");
        assert_eq!(v1, FooV1(7, true));
    }
}

/// V2 → V1: V2 appends a new tuple position; V1 does *not* opt into
/// `allow_unknown_tags`. Decode errors because the wire is longer than the
/// type's arity and the type didn't opt in to skipping extras.
mod v2_to_v1_tuple_extra_without_allow_unknown {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    struct FooV1(#[tag(0)] u32, #[tag(1)] bool);

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV2(#[tag(0)] u32, #[tag(1)] bool, #[tag(2)] u8);

    #[test]
    fn extra_trailing_position_errors_when_not_allowed() {
        let v2 = FooV2(7, true, 42);
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let err = msgpack_tagged_deserialize::<FooV1>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("wire has 3 entries") && msg.contains("at most 2"), "got: {msg}",);
    }
}

/// V1 → V2: V2 retires a tuple position by adding its tag to `reserved(...)`
/// and dropping the field. The wire is longer than V2's arity but the excess
/// entry's tag is reserved, so V2 silently skips it on decode — parallel
/// to the named-struct `v1_to_v2_remove_field_with_reserved` case.
mod v1_to_v2_tuple_remove_field_with_reserved {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    struct FooV1(#[tag(0)] u32, #[tag(1)] bool, #[tag(2)] u8);

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(reserved(1))]
    struct FooV2(#[tag(0)] u32, #[tag(2)] u8);

    #[test]
    fn skip_retired_tuple_tag_on_decode() {
        let v1 = FooV1(7, true, 42);
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2(7, 42));
    }
}

/// V2 → V1: V2 extends a tuple *variant*'s payload with a new trailing
/// element. V1's variant carries `#[tagged(allow_unknown_tags)]`, so the
/// extra wire entry is buffered and silently discarded — mirror of
/// `v2_to_v1_tuple_extra_with_allow_unknown` but for the enum-variant path.
mod v2_to_v1_tuple_variant_extra_with_allow_unknown {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        #[tagged(allow_unknown_tags)]
        Carry(#[tag(0)] u32, #[tag(1)] bool),
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        Carry(#[tag(0)] u32, #[tag(1)] bool, #[tag(2)] u8),
    }

    #[test]
    fn extra_payload_position_skipped_when_allowed() {
        let v2 = FooV2::Carry(7, true, 42);
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let v1: FooV1 = msgpack_tagged_deserialize(&bytes).expect("decode V1");
        assert_eq!(v1, FooV1::Carry(7, true));
    }
}

/// V1 → V2: V2 retires a tuple-variant payload position by adding its tag
/// to the variant-level `#[tagged(reserved(...))]`. Same reserved-skip
/// semantics as the plain tuple-struct case, just reached through an
/// enum variant.
mod v1_to_v2_tuple_variant_remove_field_with_reserved {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        Carry(#[tag(0)] u32, #[tag(1)] bool, #[tag(2)] u8),
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        #[tagged(reserved(1))]
        Carry(#[tag(0)] u32, #[tag(2)] u8),
    }

    #[test]
    fn skip_retired_payload_position_on_decode() {
        let v1 = FooV1::Carry(7, true, 42);
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2::Carry(7, 42));
    }
}

// ============================================================================
// Schema-evolution / cross-version tests for *struct* variants. Most of the
// payload-side plumbing is shared with plain named structs
// (`TaggedProductMapAccess`, `Product.field_for`), so these primarily verify
// that wrapping in an enum doesn't break the schema-evolution semantics.
// ============================================================================

/// V1 → V2: V2 adds a new field on a struct-variant payload marked
/// `#[serde(default)]`. V1's wire doesn't carry that field's tag; V2's
/// decode yields `Ok(None)` for the missing key and serde-derive's standard
/// default machinery fills it in.
mod v1_to_v2_struct_variant_add_field_with_default {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
        },
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
            #[tag(1)]
            #[serde(default)]
            b: Vec<u8>,
        },
    }

    #[test]
    fn missing_payload_field_uses_default() {
        let v1 = FooV1::Carry { a: 7 };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2::Carry { a: 7, b: Vec::default() });
    }
}

/// V1 → V2: V2 declares the same payload fields in a different source order.
/// The wire is tag-keyed, not source-position-keyed, so reordering is
/// invisible.
mod v1_to_v2_struct_variant_reorder_fields {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
            #[tag(1)]
            b: bool,
        },
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        Carry {
            #[tag(1)]
            b: bool,
            #[tag(0)]
            a: u32,
        },
    }

    #[test]
    fn reorder_only_roundtrips() {
        let v1 = FooV1::Carry { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2::Carry { a: 7, b: true });
    }
}

/// V1 → V2: V2 renames a payload field's Rust ident while keeping the same
/// tag. Field names never reach the wire — only tags — so the rename is
/// invisible.
mod v1_to_v2_struct_variant_rename_field {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            field_a: u32,
        },
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            renamed_a: u32,
        },
    }

    #[test]
    fn rename_keeping_same_tag_roundtrips() {
        let v1 = FooV1::Carry { field_a: 7 };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2::Carry { renamed_a: 7 });
    }
}

/// V2 → V1: V2 drops a payload field that V1 still requires. V1's decode
/// surfaces a `missing field` error from serde-derive.
mod v2_to_v1_struct_variant_required_field_missing {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
            #[tag(1)]
            b: bool,
        },
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
        },
    }

    #[test]
    fn required_payload_field_missing_errors() {
        let v2 = FooV2::Carry { a: 7 };
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let err = msgpack_tagged_deserialize::<FooV1>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("missing field") || msg.contains("`b`"), "got: {msg}");
    }
}

/// V2 → V1: V2 adds a new payload field. V1's variant carries
/// `#[tagged(allow_unknown_tags)]`, so the extra entry is silently skipped.
mod v2_to_v1_struct_variant_extra_field_with_allow_unknown {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        #[tagged(allow_unknown_tags)]
        Carry {
            #[tag(0)]
            a: u32,
        },
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
            #[tag(1)]
            b: bool,
        },
    }

    #[test]
    fn unknown_payload_field_skipped_when_allowed() {
        let v2 = FooV2::Carry { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let v1: FooV1 = msgpack_tagged_deserialize(&bytes).expect("decode V1");
        assert_eq!(v1, FooV1::Carry { a: 7 });
    }
}

/// V2 → V1: V2 adds a new payload field; V1's variant does *not* opt into
/// `allow_unknown_tags`. V1 errors on the unknown payload tag.
mod v2_to_v1_struct_variant_extra_field_without_allow_unknown {
    use super::*;

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
        },
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
            #[tag(1)]
            b: bool,
        },
    }

    #[test]
    fn unknown_payload_field_errors_when_not_allowed() {
        let v2 = FooV2::Carry { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let err = msgpack_tagged_deserialize::<FooV1>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("unknown wire tag"), "got: {msg}");
    }
}

/// V1 → V2: V2 retires a payload field by adding its tag to
/// `#[tagged(reserved(...))]` on the variant. V1's wire carries the
/// retired tag; V2 silently skips it — same `TaggedProductMapAccess`
/// reserved-skip path as plain named structs, just reached through a
/// struct-variant payload.
mod v1_to_v2_struct_variant_remove_field_with_reserved {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        Carry {
            #[tag(0)]
            a: u32,
            #[tag(1)]
            b: bool,
        },
    }

    #[derive(serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        #[tagged(reserved(1))]
        Carry {
            #[tag(0)]
            a: u32,
        },
    }

    #[test]
    fn skip_retired_payload_tag_on_decode() {
        let v1 = FooV1::Carry { a: 7, b: true };
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2::Carry { a: 7 });
    }
}

// ============================================================================
// Schema-evolution / cross-version tests for enum *variant tags*. Each test
// pair encodes one version's value and decodes through the other version's
// type; a `#[tagged(on_reserved)]` or `#[tagged(on_unknown)]` unit variant
// is where retired / unknown tags land. The marker itself is the opt-in —
// no separate type-level flag is needed.
// ============================================================================

/// V2 → V1: V2 adds a new variant. V1 has a `#[tagged(on_unknown)]` unit
/// variant — V1 decodes V2's new variant as that fallback, discarding the
/// payload bytes.
mod v2_to_v1_enum_add_variant_with_on_unknown {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        A,
        #[tag(9)]
        #[tagged(on_unknown)]
        Unknown,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        A,
        #[tag(1)]
        B(u32, bool),
    }

    #[test]
    fn new_variant_decoded_as_fallback() {
        let v2 = FooV2::B(7, true);
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let v1: FooV1 = msgpack_tagged_deserialize(&bytes).expect("decode V1");
        assert_eq!(v1, FooV1::Unknown);
    }

    /// And the known variants still decode as themselves — the fallback
    /// only fires for unknown tags.
    #[test]
    fn known_variant_still_round_trips() {
        let v2 = FooV2::A;
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let v1: FooV1 = msgpack_tagged_deserialize(&bytes).expect("decode V1");
        assert_eq!(v1, FooV1::A);
    }
}

/// V2 → V1: V2 adds a new variant; V1 has no `#[tagged(on_unknown)]` marker.
/// V1 errors on the unknown variant tag — declaring a fallback variant is
/// the opt-in; without the marker there's no routing target.
mod v2_to_v1_enum_add_variant_without_on_unknown {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        A,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV2 {
        #[tag(0)]
        A,
        #[tag(1)]
        B,
    }

    #[test]
    fn unknown_variant_errors_when_no_marker() {
        let v2 = FooV2::B;
        let bytes = msgpack_tagged_serialize(&v2).expect("encode V2");
        let err = msgpack_tagged_deserialize::<FooV1>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("unknown variant tag 1"), "got: {msg}");
    }
}

/// V1 → V2: V2 retires a variant by adding its tag to `reserved(...)` and
/// dropping the declaration. V2 marks a unit variant with `#[tagged(on_reserved)]`,
/// so legacy data carrying the retired tag still decodes there.
mod v1_to_v2_enum_retire_variant_with_on_reserved {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        A,
        #[tag(1)]
        B,
    }

    #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(reserved(1))]
    enum FooV2 {
        #[tag(0)]
        A,
        #[tag(9)]
        #[tagged(on_reserved)]
        Retired,
    }

    #[test]
    fn retired_variant_decoded_as_fallback() {
        let v1 = FooV1::B;
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode V2");
        assert_eq!(v2, FooV2::Retired);
    }
}

/// V1 → V2: V2 only marks `#[tagged(on_reserved)]` (not `on_unknown`).
/// A *truly unknown* tag (not in `reserved`) still errors — this verifies
/// the two markers are independent axes, not redundant ones.
mod v1_to_v2_enum_unknown_not_in_reserved_still_errors_with_on_reserved_only {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        A,
        #[tag(2)]
        Bogus,
    }

    #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(reserved(1))]
    enum FooV2 {
        #[tag(0)]
        A,
        #[tag(9)]
        #[tagged(on_reserved)]
        Retired,
    }

    #[test]
    fn unknown_tag_outside_reserved_errors() {
        let v1 = FooV1::Bogus;
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let err = msgpack_tagged_deserialize::<FooV2>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("unknown variant tag 2"), "got: {msg}");
    }
}

/// And the symmetric case: V2 marks only `#[tagged(on_unknown)]` (not
/// `on_reserved`). A tag listed in `reserved` is *not* routed to the
/// `on_unknown` variant — the markers are strictly separate.
mod v1_to_v2_enum_reserved_tag_not_routed_to_on_unknown {
    use super::*;

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1 {
        #[tag(0)]
        A,
        #[tag(1)]
        B,
    }

    #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(reserved(1))]
    enum FooV2 {
        #[tag(0)]
        A,
        #[tag(9)]
        #[tagged(on_unknown)]
        Unknown,
    }

    #[test]
    fn reserved_tag_errors_when_only_on_unknown_is_marked() {
        let v1 = FooV1::B;
        let bytes = msgpack_tagged_serialize(&v1).expect("encode V1");
        let err = msgpack_tagged_deserialize::<FooV2>(&bytes).expect_err("decode should fail");
        let msg = err.to_string();
        assert!(msg.contains("unknown variant tag 1"), "got: {msg}");
    }
}

/// Unified fallback: both markers on a single variant — that variant
/// catches both retired and unknown tags. The simple "I don't care about
/// the distinction" shape.
mod unified_fallback_with_both_markers {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    #[serde(rename = "Foo")]
    #[tagged(reserved(1))]
    enum FooV2 {
        #[tag(0)]
        A,
        #[tag(9)]
        #[tagged(on_reserved, on_unknown)]
        Other,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV1Retired {
        #[tag(0)]
        A,
        #[tag(1)]
        Retired,
    }

    #[derive(serde::Serialize, MsgpackTagged)]
    #[serde(rename = "Foo")]
    enum FooV3New {
        #[tag(0)]
        A,
        #[tag(5)]
        Future,
    }

    #[test]
    fn retired_tag_lands_on_unified_fallback() {
        let bytes = msgpack_tagged_serialize(&FooV1Retired::Retired).expect("encode");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode");
        assert_eq!(v2, FooV2::Other);
    }

    #[test]
    fn unknown_tag_lands_on_unified_fallback() {
        let bytes = msgpack_tagged_serialize(&FooV3New::Future).expect("encode");
        let v2: FooV2 = msgpack_tagged_deserialize(&bytes).expect("decode");
        assert_eq!(v2, FooV2::Other);
    }
}

/// The fallback variant itself round-trips like any other unit variant —
/// the markers don't change its on-wire encoding.
mod fallback_variant_round_trip {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Debug)]
    enum WithFallback {
        #[tag(0)]
        Known,
        #[tag(9)]
        #[tagged(on_unknown)]
        Other,
    }

    #[test]
    fn fallback_round_trips() {
        assert_roundtrip(WithFallback::Other);
    }

    #[test]
    fn other_known_variant_still_round_trips() {
        assert_roundtrip(WithFallback::Known);
    }
}
