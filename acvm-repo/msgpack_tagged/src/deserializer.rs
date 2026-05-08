//! Tagged-map msgpack deserializer that wraps [`rmp_serde::Deserializer`].
//!
//! Mirrors [`crate::serializer::Serializer`]: every aggregate shape will
//! eventually be intercepted so an integer wire tag is translated back into
//! the serde field/variant name the [`Visitor`] expects, via the
//! [`TagRegistry`]. Currently a skeleton — every method forwards to the
//! inner `rmp_serde` deserializer, and the shapes that still need
//! interception are marked with a `TODO:` comment at the method body.
//!
//! The public entry point is [`msgpack_tagged_deserialize`], which builds
//! the registry up front via `T::register_into` and runs the bytes through
//! the wrapper.

use std::io::Read;

use rmp::Marker;
use rmp_serde::Deserializer as RmpDeserializer;
use rmp_serde::decode::ReadReader;
// `de::Deserializer` would clash with our own `Deserializer` struct below if
// pulled in via `use`; importing the `de` module instead lets us write
// `de::Deserializer` for the trait at the few sites that need it.
use serde::de::{self, Deserialize, DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};

use crate::{MsgpackTagged, TagRegistry};

/// `rmp_serde`'s decode-side error type, re-exported for our `Deserializer`
/// impl. Matches the encode-side `RmpError` re-export in `serializer.rs`.
type RmpError = rmp_serde::decode::Error;

/// Tagged-map msgpack deserializer.
///
/// Constructed internally by [`msgpack_tagged_deserialize`]; not part of the
/// public API yet — once strategy customization lands the builder will
/// expose it (mirroring the serializer's plan).
/// `R` is the underlying byte source — typically `&'de [u8]`. We hold the
/// inner deserializer as `RmpDeserializer<ReadReader<R>>` (the shape that
/// `RmpDeserializer::new` constructs) rather than as a generic
/// `RmpDeserializer<R>` so we can reach the underlying reader via
/// `inner.get_mut()`. That accessor is only defined on the
/// `Deserializer<ReadReader<_>, _>` flavor, and we need it for `Option`'s
/// peek-via-rewind dance.
pub(crate) struct Deserializer<'a, R: Read> {
    inner: RmpDeserializer<ReadReader<R>>,
    #[allow(dead_code)] // wired up as each shape's interception lands.
    registry: &'a TagRegistry,
}

/// Build the tag registry from `T::register_into`, then deserialize a value
/// of type `T` from `bytes` through a [`Deserializer`].
///
/// All tagged types are expected to be encoded in the **Tagged** strategy
/// (int-keyed maps) produced by
/// [`crate::serializer::msgpack_tagged_serialize`]. Strategy decoding (Array,
/// Named) and per-type strategy selection are follow-ups.
pub fn msgpack_tagged_deserialize<'de, T>(bytes: &'de [u8]) -> std::io::Result<T>
where
    T: Deserialize<'de> + MsgpackTagged,
{
    let registry = TagRegistry::from_type::<T>();
    let inner = RmpDeserializer::new(bytes);
    let mut deserializer = Deserializer { inner, registry: &registry };
    T::deserialize(&mut deserializer).map_err(std::io::Error::other)
}

// ============================================================================
// `serde::Deserializer` impl — every method currently forwards to the inner
// rmp_serde deserializer. The structurally-significant ones (struct, enum,
// tuple shapes, sequences, maps) need interception so integer wire tags are
// translated back to serde field/variant names; each is marked with a
// `TODO:` comment at its body.
// ============================================================================

impl<'de, 'a, 'der, R> de::Deserializer<'de> for &'der mut Deserializer<'a, R>
where
    // `R: Read` gives us `ReadReader<R>: ReadSlice<'de>` (rmp_serde's blanket
    // impl), which is what the inner `RmpDeserializer<ReadReader<R>>: Deserializer<'de>`
    // bound resolves to. `R: Clone` is the cost of admission for
    // `deserialize_option`'s peek-via-rewind dance.
    R: Read + Clone,
{
    type Error = RmpError;

    // TODO: when used with self-describing visitors (e.g. `serde_json::Value`,
    // `untagged` enums), `deserialize_any` peeks at the next msgpack token
    // and dispatches. rmp_serde routes nested values through its inner
    // deserializer, so a tagged value reachable from a self-describing
    // consumer wouldn't recurse via this wrapper. Niche today — none of our
    // ACIR types are decoded via `deserialize_any` — but flag for parity.
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_any(visitor)
    }

    // -------- primitive scalars: forward verbatim ---------------------------

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_bool(visitor)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_i8(visitor)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_i16(visitor)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_i32(visitor)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_i64(visitor)
    }
    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_i128(visitor)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_u8(visitor)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_u16(visitor)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_u32(visitor)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_u64(visitor)
    }
    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_u128(visitor)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_f32(visitor)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_f64(visitor)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_char(visitor)
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_str(visitor)
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_string(visitor)
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_bytes(visitor)
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_byte_buf(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // We need to peek at the next msgpack marker to decide nil → None
        // vs. anything else → Some. rmp_serde keeps an internal marker
        // buffer for this dance, but it's private — so we read the marker
        // via `rmp::decode::read_marker` directly against the underlying
        // reader, and on a non-nil marker we restore the reader's state so
        // the inner deserializer re-reads the marker as part of the value.
        //
        // `R: Clone` is the cost of admission for that restore. For the
        // `&[u8]`-shaped readers that the public entry function constructs
        // it's a trivial slice-handle copy.
        let reader_state_before = self.inner.get_mut().clone();
        let marker = rmp::decode::read_marker(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack marker: {e:?}")))?;
        if matches!(marker, Marker::Null) {
            visitor.visit_none()
        } else {
            // Restoring is load-bearing, not just hygiene: a msgpack
            // value's marker byte IS the first byte of the value
            // (`Some(42u32)` → `0x2a` alone; `Some(255u32)` → `0xcc 0xff`).
            // Leaving the reader past the marker would have the inner
            // deserializer's `read_marker` either misinterpret the
            // payload as a new marker or hit EOF, depending on the
            // value's shape. rmp_serde's own buffered-marker trick (its
            // private `marker: Option<Marker>` field) is unreachable from
            // here, so restore-then-recurse is the cleanest available
            // option. After the restore, `visit_some(&mut *self)`
            // recurses through this wrapper so nested tagged types inside
            // the inner value still get our interception.
            *self.inner.get_mut() = reader_state_before;
            visitor.visit_some(&mut *self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_unit(visitor)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // No fields → no tag table to consult; passthrough is fine.
        (&mut self.inner).deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // rmp_serde handles its `_ExtStruct` newtype specially (msgpack
        // extension types — timestamps and friends), reading an ext header
        // and constructing an `ExtDeserializer`. We don't model that wire
        // shape here, so let it through to inner verbatim. Every other
        // newtype passes through to the inner type on the wire (matching
        // the serializer's `serialize_newtype_struct`) — call
        // `visitor.visit_newtype_struct(&mut *self)` so the inner value's
        // deserialization recurses through this wrapper.
        if name == rmp_serde::MSGPACK_EXT_STRUCT_NAME {
            return (&mut self.inner).deserialize_newtype_struct(name, visitor);
        }
        visitor.visit_newtype_struct(&mut *self)
    }

    // -------- collection / aggregate shapes: forwarded for now; subsequent
    //          commits intercept these.

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Read the msgpack array header (length-prefixed; `read_array_len`
        // consumes the marker and returns the element count). Unlike the
        // option case, the marker is metadata — the elements come AFTER
        // it — so consuming it is correct and we don't need to restore.
        // The adapter then yields each element via `&mut *self.parent`
        // so any tagged element inside the sequence recurses through
        // this wrapper.
        let len = rmp::decode::read_array_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack array length: {e:?}")))?;
        visitor.visit_seq(TaggedAccessViaParent { parent: self, remaining: len as usize })
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // Same wire shape as `deserialize_seq` — a length-prefixed msgpack
        // array — so we share the access adapter. The added eager check
        // is that the wire length matches the tuple arity: serde's tuple
        // visitor reads exactly `len` elements and stops, so a wire that
        // claims more would silently leave trailing bytes in the stream
        // and corrupt subsequent reads.
        let actual = rmp::decode::read_array_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack array length: {e:?}")))?;
        if actual as usize != len {
            return Err(RmpError::custom(format!(
                "tuple length mismatch: type expects {len} elements, wire has {actual}",
            )));
        }
        visitor.visit_seq(TaggedAccessViaParent { parent: self, remaining: len })
    }

    // TODO: top-level tuple struct (e.g. `struct Pair(u32, bool)`) is an
    // int-keyed map on the wire — the registered `Product` uses positional
    // names ("0", "1", …). Read the map, look up each int key in the
    // `Product` to get the positional name, and route each value through
    // `&mut self` so nested tagged types recurse. Mirror of the serializer's
    // `serialize_tuple_struct`.
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Same shape as `deserialize_seq` — read the length-prefixed
        // header, then yield each entry's key+value through the parent.
        let len = rmp::decode::read_map_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack map length: {e:?}")))?;
        visitor.visit_map(TaggedAccessViaParent { parent: self, remaining: len as usize })
    }

    // TODO: the load-bearing one. Read an int-keyed msgpack map, translate
    // each integer wire tag back to its serde field name (looked up in the
    // registered `Product`, honoring `#[serde(rename)]`), and hand the
    // visitor a `MapAccess` that yields field-name strings as keys. Honor
    // `Product.allow_unknown_tags` (skip vs error on unknown tags) and
    // `Product.defaults` (fill `T::default()` when a tag is missing). Route
    // each value through `&mut self` so nested tagged types recurse. Mirror
    // of the serializer's `serialize_struct` + `TaggedSerializeProduct`.
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_struct(name, fields, visitor)
    }

    // TODO: read a 1-entry msgpack map (variant tag → payload). Look up the
    // variant by tag in the registered `Sum`, dispatch on its `VariantKind`
    // (unit / newtype / tuple / struct), and decode the payload accordingly:
    // `nil` for unit, the inner value pass-through for newtype, an
    // int-keyed payload map for tuple/struct (driven by the variant's
    // `payload` `Product`). Honor `Sum.default_on_reserved` and
    // `Sum.default_on_unknown` for lenient decode. Route payload values
    // through `&mut self` so nested tagged types recurse.
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_enum(name, variants, visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_identifier(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_ignored_any(visitor)
    }

    fn is_human_readable(&self) -> bool {
        // msgpack is a binary format. rmp_serde lets users opt into a
        // human-readable mode (`with_human_readable()`), but our wrapper
        // currently only constructs the inner deserializer with the default
        // config — and that default is binary.
        false
    }
}

/// Shared access adapter routing each yielded value through the parent
/// [`Deserializer`]. The msgpack length-prefixed header (array length for
/// sequences, map length for maps) is consumed up front in the
/// corresponding `deserialize_*` method before this adapter is built; from
/// there each element/key/value just reads its own bytes through the
/// parent, so any nested tagged values still see this wrapper's
/// interception.
///
/// Used as `SeqAccess` (e.g. `Vec<T>`, `&[T]`) and `MapAccess` (e.g.
/// `BTreeMap<K, V>`). Once `deserialize_tuple` lands, fixed-length Rust
/// tuples will share the `SeqAccess` impl too. Mirror of the serializer's
/// `TaggedSerializeViaParent`.
pub(crate) struct TaggedAccessViaParent<'der, 'a, R: Read> {
    parent: &'der mut Deserializer<'a, R>,
    remaining: usize,
}

/// Variable-length sequences and fixed-length tuples — both wire-encoded
/// as msgpack arrays. `next_element_seed` decrements `remaining` and
/// deserializes one element through the parent.
impl<'de, 'der, 'a, R> SeqAccess<'de> for TaggedAccessViaParent<'der, 'a, R>
where
    R: Read + Clone,
{
    type Error = RmpError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut *self.parent).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

/// Free-form maps. `next_key_seed` decrements `remaining` and deserializes
/// the key; `next_value_seed` deserializes the value without
/// decrementing (it pairs with the just-yielded key).
impl<'de, 'der, 'a, R> MapAccess<'de> for TaggedAccessViaParent<'der, 'a, R>
where
    R: Read + Clone,
{
    type Error = RmpError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut *self.parent).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.parent)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializer::msgpack_tagged_serialize;

    /// Round-trip `value` through `msgpack_tagged_serialize` then
    /// `msgpack_tagged_deserialize` and assert it survives unchanged. The
    /// shared shape every interception test will use as it lands.
    pub(crate) fn assert_roundtrip<T>(value: T)
    where
        T: serde::Serialize + de::DeserializeOwned + MsgpackTagged + PartialEq + std::fmt::Debug,
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
}
