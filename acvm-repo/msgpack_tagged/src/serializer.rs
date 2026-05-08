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
//! Every aggregate shape is intercepted: named structs, multi-element tuple
//! structs, sequences, fixed-length tuples, free-form maps, and all four
//! variant kinds (unit / newtype / tuple / struct). Primitives, top-level
//! newtype structs, and `Option` forward through to inner — recursing into
//! nested values via this same wrapper so any tagged value reachable from
//! the root keeps the int-keyed-map treatment.

use std::io::Write;

use rmp_serde::Serializer as RmpSerializer;
// `ser::Serializer` would clash with our own `Serializer` struct below if
// pulled in via `use`; importing the `ser` module instead lets us write
// `ser::Serializer` for the trait at the few sites that need it.
use serde::ser::{
    self, Error as _, Serialize, SerializeMap, SerializeSeq, SerializeStruct,
    SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::{MsgpackTagged, TagRegistry};

/// Tagged-map msgpack serializer.
///
/// Constructed internally by [`msgpack_tagged_serialize`]; not part of the
/// public API yet — once strategy customization lands the builder will
/// expose it.
pub(crate) struct Serializer<'a, W: Write> {
    inner: RmpSerializer<W>,
    registry: &'a TagRegistry,
}

impl<'a, W: Write> Serializer<'a, W> {
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
/// through a [`Serializer`] into a freshly-allocated `Vec<u8>`.
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
    let mut serializer = Serializer::new(&mut buf, &registry);
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

impl<'a, 'ser, W: Write> ser::Serializer for &'ser mut Serializer<'a, W> {
    type Ok = ();
    type Error = RmpError;

    type SerializeSeq = TaggedSerializeViaParent<'ser, 'a, W>;
    type SerializeTuple = TaggedSerializeViaParent<'ser, 'a, W>;
    type SerializeTupleStruct = TaggedSerializeProduct<'ser, 'a, W>;
    type SerializeTupleVariant = TaggedSerializeProduct<'ser, 'a, W>;
    type SerializeMap = TaggedSerializeViaParent<'ser, 'a, W>;
    type SerializeStruct = TaggedSerializeProduct<'ser, 'a, W>;
    type SerializeStructVariant = TaggedSerializeProduct<'ser, 'a, W>;

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

    // -------- product shapes (named struct + multi-element tuple struct) ---

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        // Same shape as a named struct on the wire — the registered `Product`
        // just uses positional wire-names ("0", "1", …) instead of field
        // idents. The adapter handles both via different trait impls on the
        // same struct.
        let product = self.product_for(name);
        assert_field_count_matches(name, len, product.fields.len());
        write_map_header(self.inner.get_mut(), product.fields.len())?;
        Ok(TaggedSerializeProduct { product, parent: self, next_position: 0 })
    }

    // -------- sum shapes: every variant becomes `{<variant_tag>: <payload>}`,
    // with the payload shape determined by `VariantKind`. Each branch looks
    // up the `Sum`, resolves the variant by name, writes a 1-entry outer map
    // header (variant tag → payload), then emits the payload appropriate to
    // the kind: `nil` for unit, the inner value pass-through for newtype, an
    // int-keyed payload map for tuple/struct.

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_variant_header(name, variant)?;
        ser::Serializer::serialize_unit(&mut *self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.write_variant_header(name, variant)?;
        // Pass-through: inner bytes go directly under the variant tag, with no
        // payload field-tag wrapping. Routing through `&mut *self` keeps any
        // nested tagged types inside `value` recursing through this wrapper.
        value.serialize(&mut *self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let v = self.write_variant_header(name, variant)?;
        assert_field_count_matches(variant, len, v.payload.fields.len());
        write_map_header(self.inner.get_mut(), v.payload.fields.len())?;
        Ok(TaggedSerializeProduct { product: v.payload, parent: self, next_position: 0 })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let v = self.write_variant_header(name, variant)?;
        assert_field_count_matches(variant, len, v.payload.fields.len());
        write_map_header(self.inner.get_mut(), v.payload.fields.len())?;
        Ok(TaggedSerializeProduct { product: v.payload, parent: self, next_position: 0 })
    }

    // -------- product shapes (named struct + multi-element tuple struct) ---

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let product = self.product_for(name);
        assert_field_count_matches(name, len, product.fields.len());

        // Write the msgpack map header directly to the underlying writer via
        // `rmp::encode`. We deliberately bypass `inner.serialize_map(...)` —
        // that returns a `SerializeMap` that owns the inner serializer for
        // its lifetime, which would prevent us from re-routing each value
        // back through *this* wrapper. Routing field *values* (not just the
        // top-level struct) through the wrapper is what makes nested tagged
        // types decode the same way as top-level ones.
        write_map_header(self.inner.get_mut(), product.fields.len())?;

        Ok(TaggedSerializeProduct { product, parent: self, next_position: 0 })
    }
}

impl<'a, W: Write> Serializer<'a, W> {
    /// Resolve a registered `Product` by serde name. Used by both
    /// `serialize_struct` and `serialize_tuple_struct`. A registry miss or a
    /// sum-shaped entry signals a real bug — `register_into` should have
    /// reached every type encoded under our wrapper, and the macro guarantees
    /// product/sum shape matches the Rust definition — so we panic loudly per
    /// the design doc rather than fabricating a synthetic shape.
    fn product_for(&self, name: &'static str) -> crate::Product {
        let entry = self.registry.get(name).unwrap_or_else(|| {
            panic!(
                "MsgpackTagged registry miss for {name:?} — the top-level `register_into` \
                 walk should have registered every reachable type. Either the type is \
                 missing `#[derive(MsgpackTagged)]` (or a hand-written impl that calls \
                 `try_insert`), or its `serde` name doesn't match the registered name \
                 (check `#[serde(rename = ...)]`)"
            )
        });
        entry.tagged().as_product().unwrap_or_else(|| {
            panic!("registry entry for {name:?} is sum-shaped but a product shape was expected")
        })
    }

    /// Resolve a registered `Variant` by enum-type name + variant name. Used
    /// by all four `serialize_*_variant` methods. A registry miss, a
    /// product-shaped entry, or an unknown variant name signals a real bug —
    /// the macro and serde-derive should agree on which name lives where.
    fn variant_for(&self, name: &'static str, variant_name: &'static str) -> crate::Variant {
        let entry = self.registry.get(name).unwrap_or_else(|| {
            panic!(
                "MsgpackTagged registry miss for enum {name:?} — the top-level \
                 `register_into` walk should have registered every reachable type"
            )
        });
        let sum = entry.tagged().as_sum().unwrap_or_else(|| {
            panic!(
                "registry entry for {name:?} is product-shaped but a sum shape was expected \
                 (a `serialize_*_variant` call landed here)"
            )
        });
        sum.variant_for(variant_name).unwrap_or_else(|| {
            panic!(
                "MsgpackTagged: variant {variant_name:?} of enum {name:?} not found in \
                 registered Sum — `#[derive(MsgpackTagged)]` and `serde::Serialize` disagree \
                 on variant names"
            )
        })
    }

    /// Write the outer `{variant_tag: <payload>}` map header common to all
    /// four variant shapes — looks up the variant, writes a 1-entry msgpack
    /// map header, and writes the variant tag as the map key. Returns the
    /// resolved variant so callers can use its `payload` for the rest of the
    /// shape (a payload map for tuple/struct, the inner value for newtype,
    /// `nil` for unit).
    fn write_variant_header(
        &mut self,
        name: &'static str,
        variant_name: &'static str,
    ) -> Result<crate::Variant, RmpError> {
        let v = self.variant_for(name, variant_name);
        write_map_header(self.inner.get_mut(), 1)?;
        ser::Serializer::serialize_u8(&mut *self, v.tag)?;
        Ok(v)
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

/// Assert that serde's reported field count matches the registered
/// `Product`'s — both should drop the same skipped fields, so they should
/// always agree. A mismatch is a real misconfiguration: the most common
/// cause is a `PhantomData<T>` or `#[tag(skip)]` field that's missing a
/// paired `#[serde(skip)]`, which would otherwise fail later with a
/// confusing "field not found in registered Product" error from a tag
/// lookup. The assert surfaces it earlier, with a more actionable message.
fn assert_field_count_matches(name: &str, serde_len: usize, product_len: usize) {
    assert_eq!(
        serde_len, product_len,
        "MsgpackTagged: serde reports {serde_len} fields for {name:?} but the registered \
         Product carries {product_len}. The macro and `serde::Serialize` disagree on \
         which fields are on the wire — typically because a `PhantomData<T>` or \
         `#[tag(skip)]` field is missing a paired `#[serde(skip)]`",
    );
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

/// Adapter for product shapes — both named structs and multi-element tuple
/// structs go through here. The two trait impls below differ only in how
/// they resolve a serde call to a wire tag: named-struct calls carry a
/// field-name string, tuple-struct calls carry an implicit position counter.
/// The map header is already written in the corresponding `serialize_*`
/// method before this adapter is constructed; from there each
/// `serialize_field` call appends a `(tag, value)` pair to the writer
/// through the parent [`Serializer`], so any nested tagged
/// value in `value` recurses through the wrapper instead of falling through
/// to `rmp_serde`'s default positional-array struct encoding.
///
/// `next_position` is only consulted by the [`SerializeTupleStruct`] impl;
/// the [`SerializeStruct`] impl ignores it.
pub(crate) struct TaggedSerializeProduct<'ser, 'a, W: Write> {
    product: crate::Product,
    parent: &'ser mut Serializer<'a, W>,
    next_position: usize,
}

impl<'ser, 'a, W: Write> TaggedSerializeProduct<'ser, 'a, W> {
    /// Emit one `(tag, value)` map entry through the parent serializer. The
    /// outer map header was written upfront in the corresponding
    /// `serialize_*` method, so each entry is just two more values appended
    /// back-to-back: the integer tag (the map key) followed by the field
    /// value. Routing the value through `&mut *self.parent` keeps any
    /// nested tagged types in `value` recursing through the wrapper.
    fn serialize_tagged<T>(&mut self, tag: u8, value: &T) -> Result<(), RmpError>
    where
        T: ?Sized + Serialize,
    {
        ser::Serializer::serialize_u8(&mut *self.parent, tag)?;
        value.serialize(&mut *self.parent)
    }
}

/// Named-field struct (`struct Foo { a: u32, b: bool }`). Each
/// `serialize_field(name, value)` call resolves `name` against the
/// registered `Product` (honoring `#[serde(rename)]`) to derive the wire
/// tag, then writes `tag` and `value` through the parent.
impl<'ser, 'a, W: Write> SerializeStruct for TaggedSerializeProduct<'ser, 'a, W> {
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
        self.serialize_tagged(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // msgpack maps are length-prefixed, not terminated.
        Ok(())
    }
}

/// Multi-element tuple struct (`struct Pair(u32, bool)`). serde calls
/// `serialize_field(value)` once per element in source-declaration order
/// without supplying a name, so we keep an internal position counter and
/// look up the wire tag for the source position (the registered `Product`
/// uses positional names `"0"`, `"1"`, … as wire-name strings). Resolving by
/// position lets `#[tag(N)]`-reordered tuple structs (e.g.
/// `struct Triple(#[tag(2)] u32, #[tag(0)] bool, #[tag(1)] u8)`) emit each
/// field under the right wire tag even though the calls arrive in source
/// order.
///
/// Note on wire ordering: entries are emitted in serde's *call order*, which
/// is source-declaration order — not necessarily tag-ascending. Same as the
/// `SerializeStruct` impl above. Tightening to tag-ascending for
/// determinism is a known follow-up that affects both impls and would
/// require buffering field bytes; doing it consistently for both shapes is
/// out of scope for this commit.
impl<'ser, 'a, W: Write> SerializeTupleStruct for TaggedSerializeProduct<'ser, 'a, W> {
    type Ok = ();
    type Error = RmpError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let position = self.next_position;
        self.next_position += 1;
        // Wire-name strings are positional ("0", "1", …) — produced by the
        // macro from `position.to_string()` lifted into a `&'static str`
        // const. We allocate a fresh `String` per call to look it up; for
        // the small (typically 2–5) field counts of tuple structs this is
        // not in any hot path.
        let position_name = position.to_string();
        let tag = self.product.tag_for(&position_name).ok_or_else(|| {
            RmpError::custom(format!(
                "MsgpackTagged: tuple-struct position {position} not found in registered \
                 Product — the macro's emitted `Product` has fewer fields than serde is \
                 trying to serialize"
            ))
        })?;
        self.serialize_tagged(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Tuple variant payload (`enum E { ... Pair(u32, bool) }`). Same payload
/// shape and tag-resolution rule as a top-level tuple struct — the variant's
/// `payload` `Product` uses positional names (`"0"`, `"1"`, …) and the inner
/// payload map header was written upfront in `serialize_tuple_variant`. The
/// outer `{variant_tag: ...}` map's only entry is the payload, so end() just
/// finishes the inner map; nothing else needs writing.
impl<'ser, 'a, W: Write> SerializeTupleVariant for TaggedSerializeProduct<'ser, 'a, W> {
    type Ok = ();
    type Error = RmpError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let position = self.next_position;
        self.next_position += 1;
        let position_name = position.to_string();
        let tag = self.product.tag_for(&position_name).ok_or_else(|| {
            RmpError::custom(format!(
                "MsgpackTagged: tuple-variant position {position} not found in registered \
                 Variant payload"
            ))
        })?;
        self.serialize_tagged(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Struct variant payload (`enum E { ... Named { a: u32, b: bool } }`). Same
/// payload shape and tag-resolution rule as a top-level named struct.
impl<'ser, 'a, W: Write> SerializeStructVariant for TaggedSerializeProduct<'ser, 'a, W> {
    type Ok = ();
    type Error = RmpError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let tag = self.product.tag_for(key).ok_or_else(|| {
            RmpError::custom(format!(
                "MsgpackTagged: field {key:?} not found in registered Variant payload — \
                 `#[derive(MsgpackTagged)]` and `serde::Serialize` disagree on variant \
                 field names (check `#[serde(rename = ...)]`)"
            ))
        })?;
        self.serialize_tagged(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Stateless pass-through adapter shared by every shape whose only job is
/// to route element/key/value calls back through the parent
/// [`Serializer`]. The msgpack header (array length or map
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
    parent: &'ser mut Serializer<'a, W>,
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
        #[allow(dead_code)]
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
        let value =
            StructVariantWithNested::Carry { inner: Pair { first: 8, second: true }, count: 3 };
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
}
