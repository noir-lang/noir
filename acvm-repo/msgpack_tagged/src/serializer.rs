//! Tagged-map msgpack serializer that wraps [`rmp_serde::Serializer`].
//!
//! Most of `serde::Serializer`'s methods are forwarded to the inner
//! `rmp_serde` serializer unchanged — we only intercept the structurally
//! significant calls (`serialize_struct`, the variant methods, etc.) and
//! re-emit them as integer-keyed msgpack maps using the [`TagRegistry`].
//!
//! The public entry point is [`msgpack_tagged_serialize`], which builds the
//! registry up front via `T::register_into` and runs the value through the
//! wrapper. The wrapper in turn writes to a `Vec<u8>` we hand back to the
//! caller.
//!
//! Currently intercepts the named-struct shape end-to-end (with nested
//! recursion through the wrapper). Every other shape — sequences, maps,
//! tuples, tuple structs, the four variant kinds — still forwards to the
//! inner `rmp_serde` serializer and is marked with a `TODO:` comment at the
//! method body explaining what each one needs.

use std::io::Write;

use rmp_serde::Serializer as RmpSerializer;
use serde::ser::{
    Error as _, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple, Serializer,
};

use crate::{MsgpackTagged, TagRegistry};

/// Tagged-map msgpack serializer.
///
/// Constructed internally by [`msgpack_tagged_serialize`]; not part of the
/// public API yet — once strategy customization lands the builder will
/// expose it.
pub(crate) struct TaggedMsgpackSerializer<'a, W: Write> {
    inner: RmpSerializer<W>,
    registry: &'a TagRegistry,
}

impl<'a, W: Write> TaggedMsgpackSerializer<'a, W> {
    fn new(writer: W, registry: &'a TagRegistry) -> Self {
        // We intentionally use rmp_serde's default config (no
        // `with_struct_map`): every tagged type's `serialize_struct` /
        // variant call routes through our interception layer below, which
        // emits int-keyed maps directly. The handful of `Serializer` methods
        // we still forward to inner won't be reached by tagged types under
        // the registry's bound chain — primitives and containers don't go
        // through `serialize_struct`, and any type that does is expected to
        // be in the registry.
        Self { inner: RmpSerializer::new(writer), registry }
    }
}

/// Build the tag registry from `T::register_into`, then serialize `value`
/// through a [`TaggedMsgpackSerializer`] into a freshly-allocated `Vec<u8>`.
///
/// All tagged types currently encode in **Tagged** strategy (int-keyed maps);
/// per-type strategy customization will land as a follow-up. Untagged types
/// are forwarded to `rmp_serde`'s default encoding (with `with_struct_map`
/// applied for consistency).
pub fn msgpack_tagged_serialize<T>(value: &T) -> std::io::Result<Vec<u8>>
where
    T: ?Sized + Serialize + MsgpackTagged,
{
    let registry = TagRegistry::from_type::<T>();

    let mut buf = Vec::new();
    let mut serializer = TaggedMsgpackSerializer::new(&mut buf, &registry);
    value.serialize(&mut serializer).map_err(std::io::Error::other)?;
    Ok(buf)
}

/// `rmp_serde`'s error type, re-exported for our `serde::Serializer` impl.
type RmpError = rmp_serde::encode::Error;

// ============================================================================
// `serde::Serializer` impl — most methods forward to the inner rmp_serde
// serializer; the structurally-significant ones (struct, variants, tuple
// shapes) are intercepted to emit int-keyed maps via the registry.
// ============================================================================

impl<'a, 'ser, W: Write> Serializer for &'ser mut TaggedMsgpackSerializer<'a, W> {
    type Ok = ();
    type Error = RmpError;

    type SerializeSeq = TaggedSerializeViaParent<'ser, 'a, W>;
    type SerializeTuple = TaggedSerializeViaParent<'ser, 'a, W>;
    type SerializeTupleStruct = <&'ser mut RmpSerializer<W> as Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant = <&'ser mut RmpSerializer<W> as Serializer>::SerializeTupleVariant;
    type SerializeMap = TaggedSerializeViaParent<'ser, 'a, W>;
    type SerializeStruct = TaggedSerializeStruct<'ser, 'a, W>;
    type SerializeStructVariant =
        <&'ser mut RmpSerializer<W> as Serializer>::SerializeStructVariant;

    // -------- primitive scalars: forward verbatim ---------------------------

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i8(v)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i16(v)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i32(v)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i64(v)
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i128(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u8(v)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u16(v)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u32(v)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u64(v)
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u128(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f32(v)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_char(v)
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_str(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_none()
    }
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Re-route `Some(inner)` through ourselves so nested tagged types
        // get the int-keyed treatment too.
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        // No fields → no tag table to consult; passthrough is fine.
        self.inner.serialize_unit_struct(name)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Newtype structs pass through to the inner type — that's the
        // language-level convention we mirrored in the macro (`expand_newtype`
        // emits an empty `Tagged::Product`). Re-route through ourselves so
        // the inner type gets the tagged treatment if applicable.
        value.serialize(self)
    }

    // -------- collection / map shapes: intercepted -------------------------
    //
    // We write the array/map header directly to the underlying writer and
    // route each element/entry back through *this* wrapper via dedicated
    // adapters (`TaggedSerializeArray`, `TaggedSerializeMap`). Without this
    // interception, rmp_serde's adapters would route nested values through
    // its own inner serializer — a tagged element inside a `Vec<Tagged>` /
    // `BTreeMap<_, Tagged>` would then fall through to rmp's default
    // positional-array struct encoding instead of recursing back to our
    // int-keyed map shape.

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // msgpack arrays are length-prefixed, so we need a known length up
        // front — same constraint rmp_serde itself imposes.
        let len = len.ok_or_else(|| {
            RmpError::custom("MsgpackTagged: sequences need a known length to encode")
        })?;
        write_array_header(self.inner.get_mut(), len)?;
        Ok(TaggedSerializeViaParent { parent: self })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        write_array_header(self.inner.get_mut(), len)?;
        Ok(TaggedSerializeViaParent { parent: self })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let len = len
            .ok_or_else(|| RmpError::custom("MsgpackTagged: maps need a known length to encode"))?;
        write_map_header(self.inner.get_mut(), len)?;
        Ok(TaggedSerializeViaParent { parent: self })
    }

    // -------- tuple struct / variant / unit variant / newtype variant:
    //          forwarded for now; subsequent commits intercept these.

    // TODO: top-level tuple struct (e.g. `struct Pair(u32, bool)`) — the macro
    // emits a `Product` with positional names ("0", "1", …). Reuse the
    // struct-style adapter: look up the `Product`, write the map header, and
    // route each element through `&mut self` under the matching positional tag.
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.inner.serialize_tuple_struct(name, len)
    }

    // TODO: tuple variants (`VariantKind::Tuple`) — emit
    // `{<variant_tag>: {0: ..., 1: ...}}`. Look up the type's `Sum`, resolve
    // the variant by name, write an outer 1-entry map header, then a payload
    // map driven by the variant's `payload` `Product` (positional names).
    // Needs a new adapter that routes each element through `&mut self`.
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.inner.serialize_tuple_variant(name, variant_index, variant, len)
    }

    // TODO: unit variants (`VariantKind::Unit`) — emit `{<variant_tag>: nil}`.
    // rmp_serde's default encodes a unit variant as the variant *name string*,
    // which is the wrong shape for us. Look up the type's `Sum`, resolve the
    // variant by name, and write a 1-entry map `tag → nil`.
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_variant(name, variant_index, variant)
    }

    // TODO: newtype variants (`VariantKind::Newtype`) — emit
    // `{<variant_tag>: <inner bytes>}` with no field-level tag. Look up the
    // `Sum`, write a 1-entry map header, then route the inner value through
    // `&mut self` so any nested tagged types in the payload keep recursing.
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.inner.serialize_newtype_variant(name, variant_index, variant, value)
    }

    // TODO: struct variants (`VariantKind::Struct`) — emit
    // `{<variant_tag>: {<field_tag>: ..., ...}}`. Look up the `Sum`, resolve
    // the variant by name, write an outer 1-entry map header, then a payload
    // map driven by the variant's `payload` `Product` (named fields).
    // Needs a new adapter that routes each value through `&mut self`.
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.inner.serialize_struct_variant(name, variant_index, variant, len)
    }

    // -------- the one we actually intercept this commit ---------------------

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // Look up the type's `Product` in the registry and start an int-keyed
        // msgpack map sized to its tagged-field count. If the registry has no
        // entry, that signals a real bug — `register_into` should have
        // visited every `MsgpackTagged` type reachable from the root — so we
        // panic loudly per the design doc.
        let entry = self.registry.get(name).unwrap_or_else(|| {
            panic!(
                "MsgpackTagged registry miss for {name:?} during serialize_struct — \
                 the top-level `register_into` walk should have registered every \
                 reachable type. Either the type is missing `#[derive(MsgpackTagged)]` \
                 (or a hand-written impl that calls `try_insert`), or its `serde` name \
                 doesn't match the registered name (check `#[serde(rename = ...)]`)"
            )
        });
        let product = entry.tagged().as_product().unwrap_or_else(|| {
            panic!("registry entry for {name:?} is sum-shaped but `serialize_struct` was called")
        });
        // TODO: tighten to `assert_eq!(len, product.fields.len(), ...)`
        // once the variant/tuple paths are wrapped and we've stress-tested
        // the invariant against `#[tag(skip)]` / `PhantomData` fixtures.
        // Auto-skipped fields are absent from both `len` and `product.fields`,
        // so they should always agree — the assert would catch any future
        // macro/serde-derive divergence.
        let _ = len;

        // Write the msgpack map header directly to the underlying writer via
        // `rmp::encode`. We deliberately bypass `inner.serialize_map(...)` —
        // that returns a `SerializeMap` that owns the inner serializer for
        // its lifetime, which would prevent us from re-routing each value
        // back through *this* wrapper. Routing field *values* (not just the
        // top-level struct) through the wrapper is what makes nested tagged
        // types decode the same way as top-level ones.
        write_map_header(self.inner.get_mut(), product.fields.len())?;

        Ok(TaggedSerializeStruct { product, parent: self })
    }
}

/// Write a msgpack array header (`fixarray` / `array16` / `array32` depending
/// on `len`) directly to the underlying writer. Used by sequences and tuples.
fn write_array_header<W: Write>(writer: &mut W, len: usize) -> Result<(), RmpError> {
    let len_u32: u32 =
        len.try_into().map_err(|_| RmpError::custom("array length doesn't fit in u32"))?;
    rmp::encode::write_array_len(writer, len_u32)
        .map_err(|e| RmpError::custom(format!("failed to write msgpack array header: {e}")))?;
    Ok(())
}

/// Write a msgpack map header (`fixmap` / `map16` / `map32` depending on
/// `len`) directly to the underlying writer. Used by structs, maps, and the
/// variant shapes once those land.
fn write_map_header<W: Write>(writer: &mut W, len: usize) -> Result<(), RmpError> {
    let len_u32: u32 =
        len.try_into().map_err(|_| RmpError::custom("map length doesn't fit in u32"))?;
    rmp::encode::write_map_len(writer, len_u32)
        .map_err(|e| RmpError::custom(format!("failed to write msgpack map header: {e}")))?;
    Ok(())
}

/// `SerializeStruct` adapter that emits each `serialize_field(name, value)`
/// call as an `(int_key, value)` map entry — looking up `name`'s tag in the
/// registered `Product` to derive the int key. Both the integer key and the
/// value are serialized *back through* the parent [`TaggedMsgpackSerializer`],
/// so a nested tagged value gets the same int-keyed-map treatment instead of
/// falling through to `rmp_serde`'s default positional-array struct encoding.
pub(crate) struct TaggedSerializeStruct<'ser, 'a, W: Write> {
    product: crate::Product,
    parent: &'ser mut TaggedMsgpackSerializer<'a, W>,
}

impl<'ser, 'a, W: Write> SerializeStruct for TaggedSerializeStruct<'ser, 'a, W> {
    type Ok = ();
    type Error = RmpError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let tag = self.product.tag_for(key).ok_or_else(|| {
            RmpError::custom(format!(
                "MsgpackTagged: field {key:?} not found in registered Product — \
                 this struct's `#[derive(MsgpackTagged)]` and `serde::Serialize` \
                 disagree on field names (check `#[serde(rename = ...)]`)",
            ))
        })?;
        // The map header was already written in `serialize_struct`. Each
        // entry is just two more values written back-to-back: the integer
        // tag (the map key) and the field value. Both go through the
        // wrapper so that any nested tagged types in `value` recurse
        // correctly.
        Serializer::serialize_u8(&mut *self.parent, tag)?;
        value.serialize(&mut *self.parent)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // msgpack maps are length-prefixed, not terminated, so there's
        // nothing left to write at end-of-struct.
        Ok(())
    }
}

/// Stateless pass-through adapter shared by every shape whose only job is
/// to route element/key/value calls back through the parent
/// [`TaggedMsgpackSerializer`]. The msgpack header (array length or map
/// length) is written upfront in the corresponding `serialize_*` method
/// before the adapter is constructed; from there each entry is just one or
/// two more values appended to the writer through the wrapper, so any
/// tagged value nested inside still gets the int-keyed-map treatment.
///
/// Used as `SerializeSeq` (e.g. `Vec<T>`), `SerializeTuple` (fixed-length
/// Rust tuples), and `SerializeMap` (e.g. `BTreeMap<K, V>`). Struct shapes
/// have their own adapter ([`TaggedSerializeStruct`]) because they carry
/// the [`Product`](crate::Product) needed to translate field names into
/// integer tags.
pub(crate) struct TaggedSerializeViaParent<'ser, 'a, W: Write> {
    parent: &'ser mut TaggedMsgpackSerializer<'a, W>,
}

/// Variable-length sequences (`Vec<T>`, `&[T]`, …). Each element recurses
/// through the parent so tagged elements stay int-keyed.
impl<'ser, 'a, W: Write> SerializeSeq for TaggedSerializeViaParent<'ser, 'a, W> {
    type Ok = ();
    type Error = RmpError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.parent)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // msgpack arrays/maps are length-prefixed, not terminated — nothing
        // to write here.
        Ok(())
    }
}

/// Fixed-length Rust tuples (`(A, B)`, `(A, B, C)`, …). Same wire shape as a
/// sequence — msgpack has one length-prefixed array, regardless of whether
/// the source was variable- or fixed-length on the Rust side.
impl<'ser, 'a, W: Write> SerializeTuple for TaggedSerializeViaParent<'ser, 'a, W> {
    type Ok = ();
    type Error = RmpError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.parent)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Free-form maps (`BTreeMap<K, V>` and friends). Both keys and values are
/// routed through the parent. Routing keys is mostly a no-op for the common
/// primitive-key case (the wrapper forwards primitives to inner verbatim),
/// but it keeps the door open for tagged keys without a special case here.
impl<'ser, 'a, W: Write> SerializeMap for TaggedSerializeViaParent<'ser, 'a, W> {
    type Ok = ();
    type Error = RmpError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.parent)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.parent)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MsgpackTagged;
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
                assert!(
                    matches!(k, Value::Integer(_)),
                    "element {i} key {k:?} should be an integer",
                );
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
}
