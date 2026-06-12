//! Integration tests for the public encode path
//! ([`msgpack_tagged::msgpack_tagged_serialize`]).
//!
//! These exercise the wrapper end-to-end: each test serializes a value and
//! inspects the resulting msgpack bytes via `rmpv::Value`, so we're
//! asserting on the actual wire shape rather than on internal serializer
//! state. Lives under `tests/` rather than `#[cfg(test)]` inside
//! `src/serializer.rs` to keep that source file readable.

#![allow(dead_code)]

use msgpack_tagged::{MsgpackTagged, msgpack_tagged_serialize};
use rmpv::Value;

/// Minimal tagged struct: two fields with non-trivial tags (skipping 0 to
/// catch any "tags happen to match positions" coincidence).
#[derive(serde::Serialize, MsgpackTagged)]
struct Pair {
    #[tag(2)]
    first: u32,
    #[tag(5)]
    second: bool,
}

fn decode_msgpack(bytes: &[u8]) -> Value {
    rmpv::decode::read_value(&mut &bytes[..]).expect("valid msgpack")
}

/// `serialize_struct` emits an int-keyed map; the keys are the field
/// tags from the registry, not the field names.
#[test]
fn named_struct_encodes_as_int_keyed_map() {
    let value = Pair { first: 7, second: true };
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 2);

    // Entries are emitted in tag-ascending source order. Both keys must
    // be integers — proves we're not falling through to rmp_serde's
    // string-keyed path.
    for (k, _) in &entries {
        assert!(matches!(k, Value::Integer(_)), "key {k:?} should be an integer");
    }

    // Tag 2 → first → 7
    let (_, v) = &entries[0];
    assert_eq!(entries[0].0.as_u64(), Some(2));
    assert_eq!(v.as_u64(), Some(7));

    // Tag 5 → second → true
    let (_, v) = &entries[1];
    assert_eq!(entries[1].0.as_u64(), Some(5));
    assert_eq!(v.as_bool(), Some(true));
}

/// Nested tagged types: the outer `Outer` field of type `Pair` recurses
/// through our wrapper, so the inner bytes are also int-keyed.
#[derive(serde::Serialize, MsgpackTagged)]
struct Outer {
    #[tag(0)]
    nested: Pair,
    #[tag(1)]
    flag: u8,
}

#[test]
fn nested_tagged_struct_recurses_through_wrapper() {
    let value = Outer { nested: Pair { first: 1, second: false }, flag: 9 };
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0.as_u64(), Some(0));
    let Value::Map(inner) = &entries[0].1 else {
        panic!("inner `nested` should also be an int-keyed map, got {:?}", entries[0].1);
    };
    assert_eq!(inner.len(), 2);
    for (k, _) in inner {
        assert!(matches!(k, Value::Integer(_)), "inner key {k:?} should be an integer");
    }
}

/// `Vec<Pair>` exercises the `serialize_seq` adapter: each element must
/// recurse through the wrapper so it lands as an int-keyed map, not as
/// rmp_serde's default positional-array struct encoding. Without the
/// adapter, each `Pair` would decode as `Array([Integer, Boolean])`.
#[test]
fn vec_of_tagged_recurses_each_element_through_wrapper() {
    let value: Vec<Pair> = vec![
        Pair { first: 1, second: true },
        Pair { first: 2, second: false },
        Pair { first: 3, second: true },
    ];
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Array(elements) = decoded else {
        panic!("expected msgpack array, got {decoded:?}");
    };
    assert_eq!(elements.len(), 3);
    for (i, element) in elements.iter().enumerate() {
        let Value::Map(entries) = element else {
            panic!("element {i} should be an int-keyed map, got {element:?}");
        };
        assert_eq!(entries.len(), 2);
        for (k, _) in entries {
            assert!(matches!(k, Value::Integer(_)), "element {i} key {k:?} should be an integer",);
        }
    }
}

/// Empty sequences still produce a valid msgpack array with length 0,
/// not nil — the encoder shouldn't short-circuit on `len == 0`.
#[test]
fn empty_vec_encodes_as_zero_length_array() {
    let value: Vec<Pair> = vec![];
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let Value::Array(elements) = decoded else {
        panic!("expected msgpack array, got {decoded:?}");
    };
    assert!(elements.is_empty());
}

/// Rust tuples go through `serialize_tuple` (fixed-length). The
/// tagged element should still recurse — same fix as `Vec<Tagged>`.
#[test]
fn tuple_with_tagged_element_recurses_through_wrapper() {
    let value: (Pair, u32) = (Pair { first: 4, second: false }, 99);
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Array(elements) = decoded else {
        panic!("expected msgpack array, got {decoded:?}");
    };
    assert_eq!(elements.len(), 2);
    let Value::Map(pair_entries) = &elements[0] else {
        panic!("first tuple element (Pair) should be an int-keyed map, got {:?}", elements[0]);
    };
    for (k, _) in pair_entries {
        assert!(matches!(k, Value::Integer(_)), "pair key {k:?} should be an integer");
    }
    assert_eq!(elements[1].as_u64(), Some(99), "second tuple element is the bare u32");
}

/// Multi-element tuple struct with implicit positional tags
/// (`#[derive]` synthesizes `(0, "0")`, `(1, "1")`). On the wire we
/// expect an int-keyed map, *not* an array — that's the int-keyed-map
/// shape we promise for any tagged product.
#[derive(serde::Serialize, MsgpackTagged)]
struct PositionalTriple(u32, bool, u8);

#[test]
fn tuple_struct_encodes_as_int_keyed_map() {
    let value = PositionalTriple(7, true, 9);
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].0.as_u64(), Some(0));
    assert_eq!(entries[0].1.as_u64(), Some(7));
    assert_eq!(entries[1].0.as_u64(), Some(1));
    assert_eq!(entries[1].1.as_bool(), Some(true));
    assert_eq!(entries[2].0.as_u64(), Some(2));
    assert_eq!(entries[2].1.as_u64(), Some(9));
}

/// Explicit `#[tag(N)]` on each element reorders the wire tags relative
/// to source position. Each element ends up under the correct tag —
/// position-0 element under tag 2, position-1 under tag 0, etc. —
/// proving the position counter resolves through the `Product`'s
/// positional names rather than blindly using the source position as
/// the wire tag.
#[derive(serde::Serialize, MsgpackTagged)]
struct ReorderedTriple(#[tag(2)] u32, #[tag(0)] bool, #[tag(1)] u8);

#[test]
fn tuple_struct_with_explicit_tags_emits_under_the_right_wire_tag() {
    let value = ReorderedTriple(7, true, 9);
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 3);

    // serde calls in source order, so the *on-wire entry order* is
    // source-declaration order — but the *tag* on each entry reflects
    // the explicit `#[tag(N)]`. Match (tag, value) pairs by tag so the
    // assertion isn't sensitive to call order vs. tag-ascending order.
    let by_tag: std::collections::BTreeMap<u64, &Value> =
        entries.iter().map(|(k, v)| (k.as_u64().expect("tag is integer"), v)).collect();
    assert_eq!(by_tag[&2].as_u64(), Some(7), "element 0 (u32 7) → tag 2");
    assert_eq!(by_tag[&0].as_bool(), Some(true), "element 1 (bool true) → tag 0");
    assert_eq!(by_tag[&1].as_u64(), Some(9), "element 2 (u8 9) → tag 1");
}

/// Tuple-struct fields recurse through the wrapper just like named
/// struct fields — a tagged element nested in a tuple struct still gets
/// the int-keyed-map treatment.
#[derive(serde::Serialize, MsgpackTagged)]
struct TupleWithNested(Pair, u8);

#[test]
fn tuple_struct_with_nested_tagged_element_recurses_through_wrapper() {
    let value = TupleWithNested(Pair { first: 1, second: false }, 42);
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0.as_u64(), Some(0));
    let Value::Map(inner) = &entries[0].1 else {
        panic!("element 0 (`Pair`) should be an int-keyed map, got {:?}", entries[0].1);
    };
    assert_eq!(inner.len(), 2);
    assert_eq!(entries[1].0.as_u64(), Some(1));
    assert_eq!(entries[1].1.as_u64(), Some(42));
}

/// `BTreeMap<_, Pair>` exercises the `serialize_map` adapter: every value
/// must recurse through the wrapper. Keys (here `u8`) forward through the
/// wrapper to inner verbatim — they're scalar primitives.
#[test]
fn btree_map_with_tagged_values_recurses_each_value_through_wrapper() {
    use std::collections::BTreeMap;
    let mut value: BTreeMap<u8, Pair> = BTreeMap::new();
    value.insert(10, Pair { first: 11, second: true });
    value.insert(20, Pair { first: 22, second: false });
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 2);
    // BTreeMap iterates in key order, so the on-the-wire order is
    // deterministic — ascending by key.
    assert_eq!(entries[0].0.as_u64(), Some(10));
    assert_eq!(entries[1].0.as_u64(), Some(20));
    for (i, (_, v)) in entries.iter().enumerate() {
        let Value::Map(pair_entries) = v else {
            panic!("entry {i} value should be an int-keyed map, got {v:?}");
        };
        assert_eq!(pair_entries.len(), 2);
        for (k, _) in pair_entries {
            assert!(
                matches!(k, Value::Integer(_)),
                "entry {i} pair key {k:?} should be an integer",
            );
        }
    }
}

/// Mixed-shape enum exercising every `VariantKind`: `Empty` is unit,
/// `Wrap` is a newtype carrying a primitive, `Pair` is a tuple variant
/// with two payload fields, `Named` is a struct variant with named
/// fields. Each variant's wire tag is non-zero so we don't accidentally
/// pass a "tag matches default" coincidence.
#[derive(serde::Serialize, MsgpackTagged)]
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

fn single_entry_map(value: &Value) -> (u64, &Value) {
    let Value::Map(entries) = value else {
        panic!("expected single-entry map, got {value:?}");
    };
    assert_eq!(entries.len(), 1, "expected exactly one entry, got {entries:?}");
    let (k, v) = &entries[0];
    (k.as_u64().expect("variant tag should be an integer"), v)
}

/// Unit variants encode as `{<variant_tag>: nil}` — *not* the variant
/// name string that rmp_serde's default would produce.
#[test]
fn unit_variant_encodes_as_variant_tag_with_nil_payload() {
    let bytes = msgpack_tagged_serialize(&Mixed::Empty).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let (tag, payload) = single_entry_map(&decoded);
    assert_eq!(tag, 1);
    assert_eq!(*payload, Value::Nil, "unit-variant payload should be nil, got {payload:?}");
}

/// Newtype variants pass the inner value through directly under the
/// variant tag — no field-tag wrapping. For a primitive inner type the
/// payload is just that primitive.
#[test]
fn newtype_variant_passes_inner_value_through() {
    let bytes = msgpack_tagged_serialize(&Mixed::Wrap(42)).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let (tag, payload) = single_entry_map(&decoded);
    assert_eq!(tag, 2);
    assert_eq!(payload.as_u64(), Some(42), "newtype payload should be the bare inner u32");
}

/// Newtype variant carrying a tagged inner type — the inner tagged
/// struct still recurses through the wrapper, so the payload is the
/// inner type's int-keyed map directly under the variant tag.
#[derive(serde::Serialize, MsgpackTagged)]
enum NewtypeWithTagged {
    #[tag(7)]
    Wrap(Pair),
}

#[test]
fn newtype_variant_with_tagged_inner_recurses_through_wrapper() {
    let value = NewtypeWithTagged::Wrap(Pair { first: 11, second: true });
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let (tag, payload) = single_entry_map(&decoded);
    assert_eq!(tag, 7);
    let Value::Map(inner) = payload else {
        panic!("newtype payload should be Pair's int-keyed map, got {payload:?}");
    };
    assert_eq!(inner.len(), 2);
    for (k, _) in inner {
        assert!(matches!(k, Value::Integer(_)), "inner key {k:?} should be an integer");
    }
}

/// Tuple variants encode as `{<variant_tag>: {0: ..., 1: ...}}` — the
/// outer 1-entry map carries the variant tag, the inner map is the
/// payload's positional `Product`.
#[test]
fn tuple_variant_encodes_as_int_keyed_payload_under_variant_tag() {
    let bytes = msgpack_tagged_serialize(&Mixed::Pair(7, true)).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let (tag, payload) = single_entry_map(&decoded);
    assert_eq!(tag, 3);
    let Value::Map(inner) = payload else {
        panic!("tuple-variant payload should be an int-keyed map, got {payload:?}");
    };
    assert_eq!(inner.len(), 2);
    assert_eq!(inner[0].0.as_u64(), Some(0));
    assert_eq!(inner[0].1.as_u64(), Some(7));
    assert_eq!(inner[1].0.as_u64(), Some(1));
    assert_eq!(inner[1].1.as_bool(), Some(true));
}

/// Struct variants encode as `{<variant_tag>: {<field_tag>: ..., ...}}`
/// — the inner payload is driven by the variant's named-field `Product`.
#[test]
fn struct_variant_encodes_as_int_keyed_payload_under_variant_tag() {
    let value = Mixed::Named { a: 99, b: false };
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let (tag, payload) = single_entry_map(&decoded);
    assert_eq!(tag, 4);
    let Value::Map(inner) = payload else {
        panic!("struct-variant payload should be an int-keyed map, got {payload:?}");
    };
    assert_eq!(inner.len(), 2);
    assert_eq!(inner[0].0.as_u64(), Some(0));
    assert_eq!(inner[0].1.as_u64(), Some(99));
    assert_eq!(inner[1].0.as_u64(), Some(1));
    assert_eq!(inner[1].1.as_bool(), Some(false));
}

/// Tagged values nested inside a tuple-variant payload still recurse
/// through the wrapper.
#[derive(serde::Serialize, MsgpackTagged)]
enum TupleVariantWithNested {
    #[tag(0)]
    Carry(Pair, u32),
}

#[test]
fn tuple_variant_payload_recurses_through_wrapper() {
    let value = TupleVariantWithNested::Carry(Pair { first: 1, second: false }, 5);
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let (_, payload) = single_entry_map(&decoded);
    let Value::Map(inner) = payload else {
        panic!("tuple-variant payload should be a map, got {payload:?}");
    };
    // Element 0 is `Pair` — must be an int-keyed map, not an array.
    let Value::Map(nested) = &inner[0].1 else {
        panic!("nested Pair should be int-keyed map, got {:?}", inner[0].1);
    };
    assert!(nested.iter().all(|(k, _)| matches!(k, Value::Integer(_))));
}

/// `#[serde(skip)]` is recognized as an alias for `#[tag(skip)]` by the
/// macro, so both sides drop the field. The serializer's field-count
/// assert checks that they agree — without the matching skip, the assert
/// would fire with a clear "macro and serde disagree" message.
#[derive(serde::Serialize, MsgpackTagged)]
struct WithSerdeSkippedField {
    #[tag(0)]
    visible: u32,
    #[serde(skip)]
    hidden: u32,
}

#[test]
fn serde_skipped_field_does_not_appear_on_the_wire() {
    let value = WithSerdeSkippedField { visible: 7, hidden: 99 };
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 1, "skipped field should be absent from the wire map");
    assert_eq!(entries[0].0.as_u64(), Some(0));
    assert_eq!(entries[0].1.as_u64(), Some(7));
}

/// `PhantomData<T>` is auto-skipped by the macro; pairing it with
/// `#[serde(skip)]` keeps serde-derive in sync. The field-count assert
/// confirms both sides agree on the wire-visible field count.
#[derive(serde::Serialize, MsgpackTagged)]
struct WithPhantomField<T: 'static> {
    #[tag(0)]
    visible: u32,
    #[serde(skip)]
    _marker: std::marker::PhantomData<T>,
}

#[test]
fn phantom_data_field_paired_with_serde_skip_does_not_appear_on_the_wire() {
    let value: WithPhantomField<u32> =
        WithPhantomField { visible: 42, _marker: std::marker::PhantomData };
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.as_u64(), Some(42));
}

/// Tagged values nested inside a struct-variant payload still recurse
/// through the wrapper.
#[derive(serde::Serialize, MsgpackTagged)]
enum StructVariantWithNested {
    #[tag(0)]
    Carry {
        #[tag(0)]
        inner: Pair,
        #[tag(1)]
        count: u8,
    },
}

#[test]
fn struct_variant_payload_recurses_through_wrapper() {
    let value = StructVariantWithNested::Carry { inner: Pair { first: 8, second: true }, count: 3 };
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);
    let (_, payload) = single_entry_map(&decoded);
    let Value::Map(inner) = payload else {
        panic!("struct-variant payload should be a map, got {payload:?}");
    };
    let Value::Map(nested) = &inner[0].1 else {
        panic!("nested Pair should be int-keyed map, got {:?}", inner[0].1);
    };
    assert!(nested.iter().all(|(k, _)| matches!(k, Value::Integer(_))));
}

// ============================================================================
// `EncodingStrategy::Array` — top-level struct/tuple-struct shape flips to
// `fixarray`; variant payloads stay int-keyed regardless.
// ============================================================================

use msgpack_tagged::{EncodingStrategy, Serializer, TagRegistry};
use serde::Serialize as _;

/// Serialize `value` through a `Serializer` configured with the given
/// default strategy. Helper to keep the encoding-strategy tests concise.
fn encode_with_default_strategy<T>(value: &T, strategy: EncodingStrategy) -> Vec<u8>
where
    T: ?Sized + serde::Serialize + MsgpackTagged,
{
    let registry = TagRegistry::from_type::<T>();
    let mut buf = Vec::new();
    let mut s = Serializer::new(&mut buf, &registry).with_default_strategy(strategy);
    value.serialize(&mut s).expect("serialize succeeds");
    buf
}

/// Under `Array` strategy a named struct emits as a positional `fixarray`,
/// fields in registered (tag-ascending) order, with no per-field tag byte.
#[test]
fn named_struct_under_array_strategy_emits_fixarray() {
    let value = Pair { first: 7, second: true };
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    let Value::Array(elements) = decoded else {
        panic!("expected msgpack array under Array strategy, got {decoded:?}");
    };
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].as_u64(), Some(7));
    assert_eq!(elements[1].as_bool(), Some(true));
}

/// Under `Array`, a tuple struct also emits as a positional `fixarray`.
/// Same shape as a named struct — the strategy is about wire shape, not
/// about which serde call lands.
#[derive(serde::Serialize, MsgpackTagged)]
struct ArrayStrategyTriple(u32, bool, u8);

#[test]
fn tuple_struct_under_array_strategy_emits_fixarray() {
    let value = ArrayStrategyTriple(7, true, 9);
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    let Value::Array(elements) = decoded else {
        panic!("expected msgpack array under Array strategy, got {decoded:?}");
    };
    assert_eq!(elements.len(), 3);
    assert_eq!(elements[0].as_u64(), Some(7));
    assert_eq!(elements[1].as_bool(), Some(true));
    assert_eq!(elements[2].as_u64(), Some(9));
}

/// Array bytes are strictly smaller than Tagged bytes for the same value —
/// no per-field tag prefix. Locks in the size-saving property the strategy
/// is meant to deliver.
#[test]
fn array_strategy_is_smaller_than_tagged_for_the_same_value() {
    let value = ArrayStrategyTriple(7, true, 9);
    let tagged = encode_with_default_strategy(&value, EncodingStrategy::Tagged);
    let array = encode_with_default_strategy(&value, EncodingStrategy::Array);
    assert!(
        array.len() < tagged.len(),
        "Array ({}) should be smaller than Tagged ({})",
        array.len(),
        tagged.len(),
    );
}

/// `ReorderedTriple(#[tag(2)] u32, #[tag(0)] bool, #[tag(1)] u8)` —
/// source order ≠ tag order. Under Array the wire must be in
/// *tag-ascending* order regardless of serde's source-order calls:
/// `[bool, u8, u32]`. The encoder buffers per-field bytes and flushes
/// sorted by tag in `TaggedSerializeProduct::finish` to make this hold.
#[test]
fn array_strategy_emits_in_tag_ascending_order_not_source_order() {
    let value = ReorderedTriple(7, true, 9);
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    let Value::Array(elements) = decoded else {
        panic!("expected msgpack array, got {decoded:?}");
    };
    assert_eq!(elements.len(), 3);
    // Tag 0 (bool true) emitted first — *not* the source-position-0 u32.
    assert_eq!(elements[0].as_bool(), Some(true));
    // Tag 1 (u8 9) emitted next.
    assert_eq!(elements[1].as_u64(), Some(9));
    // Tag 2 (u32 7) emitted last.
    assert_eq!(elements[2].as_u64(), Some(7));
}

/// Under Tagged the encoder also emits in canonical (tag-ascending)
/// order whenever source order doesn't match — the byte-determinism the
/// design doc's "TAGS define the canonical field order" section
/// promises. The decoder is order-agnostic (every wire entry carries
/// its explicit tag), but consumers reading the bytes downstream
/// (cross-implementation compatibility, hashing, cryptographic
/// commitments) rely on this property. The cost is a per-field
/// `Vec<u8>` allocation paid only for types whose tags have been
/// deliberately reordered relative to source.
#[test]
fn tagged_strategy_emits_in_tag_ascending_order_not_source_order() {
    let value = ReorderedTriple(7, true, 9);
    let bytes = msgpack_tagged_serialize(&value).expect("serialize succeeds");
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected msgpack map, got {decoded:?}");
    };
    assert_eq!(entries.len(), 3);
    // Tag-ascending order: 0 (bool), 1 (u8), 2 (u32) — not source order.
    let tags: Vec<u64> = entries.iter().map(|(k, _)| k.as_u64().expect("integer tag")).collect();
    assert_eq!(tags, vec![0, 1, 2], "tags should be flushed in ascending order");
}

/// Per-type overrides win over the default — a top-level type can be
/// `Array` while every nested type stays `Tagged`. Verifies the
/// override scoping.
#[test]
fn per_type_override_beats_default_strategy() {
    let value = Outer { nested: Pair { first: 1, second: false }, flag: 9 };

    // Default = Tagged, override Outer to Array. Outer's outer shape should
    // be an array; its inner Pair field stays Tagged (no override).
    let registry = TagRegistry::from_type::<Outer>();
    let mut buf = Vec::new();
    let mut s =
        Serializer::new(&mut buf, &registry).with_strategy::<Outer>(EncodingStrategy::Array);
    value.serialize(&mut s).expect("serialize succeeds");
    drop(s);

    let decoded = decode_msgpack(&buf);
    let Value::Array(elements) = decoded else {
        panic!("Outer should be array under override, got {decoded:?}");
    };
    assert_eq!(elements.len(), 2);
    // First element is the nested Pair, still Tagged → int-keyed map.
    let Value::Map(inner) = &elements[0] else {
        panic!("nested Pair should remain int-keyed under default Tagged, got {:?}", elements[0]);
    };
    assert!(inner.iter().all(|(k, _)| matches!(k, Value::Integer(_))));
    // Second element is the bare flag.
    assert_eq!(elements[1].as_u64(), Some(9));
}

/// Variant payloads follow the enclosing enum's strategy — under `Array`
/// the payload becomes a positional array. The *outer*
/// `{variant_tag: payload}` 1-entry map stays int-keyed regardless,
/// because variant identification is always by integer tag.
#[derive(serde::Serialize, MsgpackTagged)]
enum MixedForArray {
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
fn tuple_variant_payload_follows_array_default() {
    let value = MixedForArray::Pair(7, true);
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    // Outer: 1-entry int-keyed map {variant_tag → payload} (discriminator).
    let Value::Map(outer) = decoded else {
        panic!("variant outer should stay a map, got {decoded:?}");
    };
    assert_eq!(outer.len(), 1);
    assert_eq!(outer[0].0.as_u64(), Some(3));
    // Payload: positional array under Array strategy.
    let Value::Array(inner) = &outer[0].1 else {
        panic!(
            "tuple-variant payload should be a fixarray under Array default, got {:?}",
            outer[0].1
        );
    };
    assert_eq!(inner.len(), 2);
    assert_eq!(inner[0].as_u64(), Some(7));
    assert_eq!(inner[1].as_bool(), Some(true));
}

#[test]
fn struct_variant_payload_follows_array_default() {
    let value = MixedForArray::Named { a: 99, b: false };
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    let Value::Map(outer) = decoded else {
        panic!("variant outer should stay a map, got {decoded:?}");
    };
    assert_eq!(outer.len(), 1);
    let Value::Array(inner) = &outer[0].1 else {
        panic!(
            "struct-variant payload should be a fixarray under Array default, got {:?}",
            outer[0].1
        );
    };
    assert_eq!(inner.len(), 2);
    assert_eq!(inner[0].as_u64(), Some(99));
    assert_eq!(inner[1].as_bool(), Some(false));
}

/// And the default Tagged path keeps variant payloads as int-keyed maps —
/// nothing changed for the default-strategy case.
#[test]
fn tuple_variant_payload_stays_int_keyed_under_tagged_default() {
    let value = MixedForArray::Pair(7, true);
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Tagged);
    let decoded = decode_msgpack(&bytes);

    let Value::Map(outer) = decoded else {
        panic!("variant outer should stay a map, got {decoded:?}");
    };
    assert_eq!(outer.len(), 1);
    let Value::Map(inner) = &outer[0].1 else {
        panic!("tuple-variant payload should be a map under Tagged default, got {:?}", outer[0].1);
    };
    assert!(inner.iter().all(|(k, _)| matches!(k, Value::Integer(_))));
}

// ============================================================================
// Auto-downgrade: `Array` → `Tagged` when a product has a non-trailing
// reserved tag. Under Array the wire only carries active values
// positionally; a reserved tag with an active tag after it in tag order
// would leave the decoder misaligned on its own V2-on-V2 round-trip. The
// encoder detects this and silently flips to Tagged for the affected
// product, leaving every other type in the same serializer on its
// configured strategy.
// ============================================================================

/// Non-trailing reserved tag (1 is between active 0 and 2). Requesting
/// `Array` for this type must auto-downgrade to `Tagged` — otherwise a
/// V2-encoded buffer would not round-trip through V2's own decoder.
#[derive(serde::Serialize, MsgpackTagged)]
#[serde(rename = "AutoDowngrade")]
#[tagged(reserved(1))]
struct NonTrailingReservedForDowngrade {
    #[tag(0)]
    a: u32,
    #[tag(2)]
    c: bool,
}

#[test]
fn array_strategy_downgrades_when_reserved_is_not_trailing() {
    let value = NonTrailingReservedForDowngrade { a: 7, c: true };
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    let Value::Map(entries) = decoded else {
        panic!("expected fixmap from auto-downgrade, got {decoded:?}");
    };
    // Both active values present, keyed by their integer tags.
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0.as_u64(), Some(0));
    assert_eq!(entries[0].1.as_u64(), Some(7));
    assert_eq!(entries[1].0.as_u64(), Some(2));
    assert_eq!(entries[1].1.as_bool(), Some(true));
}

/// Strictly-trailing reserved (reserved tag is greater than every active
/// tag) keeps the requested `Array` strategy. The merged-layout decoder
/// stops at `wire_remaining == 0` before reaching the trailing reserved
/// slot, so positional alignment isn't at risk.
#[derive(serde::Serialize, MsgpackTagged)]
#[serde(rename = "TrailingReservedKeepsArray")]
#[tagged(reserved(9))]
struct TrailingReservedKeepsArray {
    #[tag(0)]
    a: u32,
    #[tag(1)]
    b: bool,
}

#[test]
fn array_strategy_preserved_when_reserved_is_strictly_trailing() {
    let value = TrailingReservedKeepsArray { a: 7, b: true };
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    let Value::Array(elements) = decoded else {
        panic!("expected fixarray (Array preserved), got {decoded:?}");
    };
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].as_u64(), Some(7));
    assert_eq!(elements[1].as_bool(), Some(true));
}

/// The downgrade applies inside variant payloads too — the variant's
/// `payload` `Product` is consulted at `begin_variant_payload`.
#[derive(serde::Serialize, MsgpackTagged)]
#[serde(rename = "AutoDowngradeEnum")]
enum AutoDowngradeEnum {
    #[tag(0)]
    #[tagged(reserved(1))]
    Carry {
        #[tag(0)]
        a: u32,
        #[tag(2)]
        c: bool,
    },
}

#[test]
fn array_strategy_downgrades_variant_payload_with_non_trailing_reserved() {
    let value = AutoDowngradeEnum::Carry { a: 7, c: true };
    let bytes = encode_with_default_strategy(&value, EncodingStrategy::Array);
    let decoded = decode_msgpack(&bytes);

    let Value::Map(outer) = decoded else {
        panic!("variant outer should stay a map, got {decoded:?}");
    };
    assert_eq!(outer.len(), 1);
    let Value::Map(payload) = &outer[0].1 else {
        panic!("variant payload should be a fixmap (downgrade kicked in), got {:?}", outer[0].1,);
    };
    assert_eq!(payload.len(), 2);
    assert_eq!(payload[0].0.as_u64(), Some(0));
    assert_eq!(payload[1].0.as_u64(), Some(2));
}
