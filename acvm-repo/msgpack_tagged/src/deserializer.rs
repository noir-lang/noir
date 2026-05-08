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

use rmp_serde::Deserializer as RmpDeserializer;
use rmp_serde::decode::ReadSlice;
// `de::Deserializer` would clash with our own `Deserializer` struct below if
// pulled in via `use`; importing the `de` module instead lets us write
// `de::Deserializer` for the trait at the few sites that need it.
use serde::de::{self, Deserialize, Visitor};

use crate::{MsgpackTagged, TagRegistry};

/// `rmp_serde`'s decode-side error type, re-exported for our `Deserializer`
/// impl. Matches the encode-side `RmpError` re-export in `serializer.rs`.
type RmpError = rmp_serde::decode::Error;

/// Tagged-map msgpack deserializer.
///
/// Constructed internally by [`msgpack_tagged_deserialize`]; not part of the
/// public API yet — once strategy customization lands the builder will
/// expose it (mirroring the serializer's plan).
pub(crate) struct Deserializer<'a, R> {
    inner: RmpDeserializer<R>,
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
    R: ReadSlice<'de>,
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

    // TODO: `Option`'s `Deserialize` impl drives the inner value via
    // `Visitor::visit_some(deserializer)`. rmp_serde passes its own inner
    // deserializer to the visitor, so any tagged value inside `Some(_)`
    // bypasses this wrapper. Peek at the next msgpack token (nil → None,
    // otherwise → Some), and call `visitor.visit_some(&mut *self)` so the
    // inner value recurses through this wrapper. Mirror of the serializer's
    // `serialize_some`, which already routes through `&mut self`.
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_option(visitor)
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

    // TODO: newtype structs pass through to the inner type on the wire (no
    // wrapping), matching the serializer's `serialize_newtype_struct`. Call
    // `visitor.visit_newtype_struct(&mut *self)` directly so the inner
    // value's deserialization recurses through this wrapper instead of
    // falling through to rmp_serde's inner.
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_newtype_struct(name, visitor)
    }

    // -------- collection / aggregate shapes: forwarded for now; subsequent
    //          commits intercept these.

    // TODO: needs a sequence adapter — each element should be deserialized
    // through `&mut self` so any tagged element inside a `Vec<Tagged>` /
    // `&[Tagged]` recurses through this wrapper instead of falling through
    // to rmp_serde's defaults. Mirror of the serializer's
    // `TaggedSerializeViaParent` for the `SerializeSeq` case.
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_seq(visitor)
    }

    // TODO: same fix as `deserialize_seq` for fixed-length Rust tuples —
    // `(Tagged, ...)` elements need to recurse through this wrapper.
    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_tuple(len, visitor)
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

    // TODO: needs a map adapter — both key and value of every entry should
    // be deserialized through `&mut self` so nested tagged values inside a
    // `BTreeMap<_, Tagged>` recurse via this wrapper.
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        (&mut self.inner).deserialize_map(visitor)
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

    /// Skeleton smoke test: a value that doesn't go through any
    /// shape-specific interception path roundtrips correctly. `Vec<u32>`
    /// uses `serialize_seq` / `deserialize_seq`, both of which currently
    /// forward to inner unchanged — so end-to-end this exercises only the
    /// wiring (registry build + wrapper construction + `T::deserialize`
    /// dispatch), not any interception.
    ///
    /// This locks in the wiring before the interception work lands, and
    /// gives us a regression target if a future change breaks the basic
    /// path.
    #[test]
    fn roundtrip_through_skeleton_for_a_shape_that_does_not_need_interception() {
        let value: Vec<u32> = vec![1, 2, 3, 4, 5];
        assert_roundtrip(value);
    }
}
