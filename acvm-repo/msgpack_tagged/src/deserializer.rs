//! Tagged-map msgpack deserializer that wraps [`rmp_serde::Deserializer`].
//!
//! Mirrors [`crate::serializer::Serializer`]: each aggregate shape that the
//! macro emits — named struct, multi-element tuple struct, sequence, tuple,
//! map, option, newtype struct — is intercepted to translate integer wire
//! tags back to the serde field/variant names the [`Visitor`] expects, via
//! the [`TagRegistry`]. The shapes that still forward to `rmp_serde` carry
//! an inline `// TODO:` at the method body.
//!
//! The public entry point is [`msgpack_tagged_deserialize`], which builds
//! the registry up front via `T::register_into` and runs the bytes through
//! the wrapper.
//!
//! ## Known gaps vs. the design doc / macro syntax
//!
//! The wrapper isn't final — the bits below are accepted by
//! `#[derive(MsgpackTagged)]` today but the deserializer doesn't model
//! them yet. Each is also flagged with an inline `// TODO:` at the
//! relevant call site.
//!
//! - **`deserialize_enum` not intercepted.** Variant tag → variant name
//!   translation + `VariantKind` dispatch isn't implemented; we fall
//!   through to `rmp_serde` which expects a different wire shape than
//!   the one our serializer produces. Until this lands, tagged enums
//!   don't round-trip through the wrapper.
//! - **`#[tagged(reserved(...))]` on the decode side.** `Product.is_reserved`
//!   exists in the registry and is checked at compile time by the macro
//!   to prevent tag reuse, but the decoder never consults it. Retired
//!   tags on the wire currently follow the same code path as wholly
//!   unknown tags — silently skipped only when `allow_unknown_tags` is
//!   set, error otherwise. The likely-intended semantic is that
//!   `reserved` tags auto-skip on decode regardless of
//!   `allow_unknown_tags`, since they're explicitly opted-in retirements.
//! - **`#[tagged(allow_unknown_tags)]` on variant payloads.** Mirrors
//!   the struct case but lives behind the missing `deserialize_enum`
//!   interception.
//! - **`#[tagged(default_on_reserved)]` / `default_on_unknown`** (enum-
//!   only). Substitute `T::default()` for retired or unknown variant
//!   tags on decode. Same dependency — `deserialize_enum` first.
//! - **`deserialize_any`.** Niche today (none of our ACIR types are
//!   decoded via self-describing visitors), but nested tagged values
//!   reached through `serde_json::Value`-style consumers wouldn't
//!   recurse through this wrapper.
//! - **Encoding strategies.** Only the **Tagged** strategy is decoded.
//!   When the serializer gains **Array** / **Named** overrides, the
//!   deserializer needs the matching shape-agnostic dispatch (peek
//!   marker, route to the right reader).

use rmp::Marker;
use rmp_serde::Deserializer as RmpDeserializer;
use rmp_serde::decode::ReadReader;
// `de::Deserializer` would clash with our own `Deserializer` struct below if
// pulled in via `use`; importing the `de` module instead lets us write
// `de::Deserializer` for the trait at the few sites that need it.
use serde::de::{self, Deserialize, DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;

use crate::{MsgpackTagged, TagRegistry};

/// `rmp_serde`'s decode-side error type, re-exported for our `Deserializer`
/// impl. Matches the encode-side `RmpError` re-export in `serializer.rs`.
type RmpError = rmp_serde::decode::Error;

/// Tagged-map msgpack deserializer.
///
/// Constructed internally by [`msgpack_tagged_deserialize`]; not part of the
/// public API yet — once strategy customization lands the builder will
/// expose it (mirroring the serializer's plan).
///
/// The inner reader is fixed to `&'de [u8]` (wrapped in rmp_serde's
/// `ReadReader`, the shape `RmpDeserializer::new` constructs). We don't
/// keep the reader generic because two interception paths need to grab a
/// snapshot of the unread byte slice — `deserialize_option`'s peek-via-
/// rewind dance and `deserialize_tuple_struct`'s buffer-by-tag — and
/// those slice tricks only work when the reader exposes the underlying
/// `&[u8]` directly. The public entry function only ever constructs the
/// `&[u8]` shape, so this isn't a real loss of generality; if/when a
/// `from_read` variant lands, that's a separate type.
pub(crate) struct Deserializer<'a, 'de> {
    inner: RmpDeserializer<ReadReader<&'de [u8]>>,
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
    let mut deserializer = Deserializer::new(bytes, &registry);
    T::deserialize(&mut deserializer).map_err(std::io::Error::other)
}

// ============================================================================
// `serde::Deserializer` impl — every method currently forwards to the inner
// rmp_serde deserializer. The structurally-significant ones (struct, enum,
// tuple shapes, sequences, maps) need interception so integer wire tags are
// translated back to serde field/variant names; each is marked with a
// `TODO:` comment at its body.
// ============================================================================

impl<'de, 'a, 'der> de::Deserializer<'de> for &'der mut Deserializer<'a, 'de> {
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
        let reader_state_before: &'de [u8] = self.inner.get_mut();
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

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // Top-level tuple structs (`struct Pair(u32, bool)`) are encoded
        // as int-keyed maps on the wire — same shape as named structs —
        // but the `Deserialize` impl calls `next_element_seed` (positional)
        // instead of `next_key_seed` (by name). We buffer every wire entry
        // as a `(tag, value-bytes)` pair upfront, then dispatch by tag in
        // `next_element_seed` via `Product.tag_for("0")`,
        // `tag_for("1")`, … so reconstruction is robust to whatever order
        // the serializer emits entries in.
        // See `TaggedTupleStructAccess`'s doc for the why.
        let product = self.product_for(name);
        let wire_len = rmp::decode::read_map_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack map length: {e:?}")))?;
        let wire_len = wire_len as usize;
        // Length-mismatch policy:
        //   * `wire_len < len` is allowed — the missing trailing positions
        //     are reported to the visitor as `Ok(None)` in `next_element_seed`,
        //     and serde-derive's `#[serde(default)]` (which it requires on
        //     all positions past the first defaulted one) fills them in. A
        //     wire missing a *required* position will still error at the
        //     visitor with `invalid length`.
        //   * `wire_len > len` is only accepted when the type opts into
        //     `#[tagged(allow_unknown_tags)]`; the trailing wire entries
        //     are simply never queried by the visitor and get discarded.
        if wire_len > len && !product.allow_unknown_tags {
            return Err(RmpError::custom(format!(
                "tuple-struct {name:?}: wire has {wire_len} entries but the type \
                 expects only {len}; opt into `#[tagged(allow_unknown_tags)]` to \
                 silently skip extras",
            )));
        }

        // Buffer each wire entry's `(tag, value-bytes)`. The slice trick:
        // snapshot the inner reader's `&[u8]` before and after reading
        // exactly one msgpack value (via `IgnoredAny`, which walks any
        // shape) — the diff is the value's byte range.
        let mut entries: SmallVec<[(u8, &'de [u8]); 4]> = SmallVec::with_capacity(wire_len);
        for _ in 0..wire_len {
            let tag: u8 = u8::deserialize(&mut *self)?;
            let before: &'de [u8] = self.inner.get_mut();
            de::IgnoredAny::deserialize(&mut *self)?;
            let after: &'de [u8] = self.inner.get_mut();
            let consumed = before.len() - after.len();
            entries.push((tag, &before[..consumed]));
        }

        visitor.visit_seq(TaggedTupleStructAccess {
            parent: self,
            product,
            entries,
            next_position: 0,
        })
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Same shape as `deserialize_seq` — read the length-prefixed
        // header, then yield each entry's key+value through the parent.
        let len = rmp::decode::read_map_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack map length: {e:?}")))?;
        visitor.visit_map(TaggedAccessViaParent { parent: self, remaining: len as usize })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // Look up the type's `Product` in the registry — it carries the
        // tag → field-name mapping we need to feed the visitor. Read the
        // msgpack map length, then hand off to `TaggedProductMapAccess`
        // which yields each entry's key as the registered field name
        // (translated from the integer wire tag) and routes each value
        // through `&mut *self.parent`. Wire-tolerance for missing tags is
        // serde-derive's job via `#[serde(default)]` — this layer stays
        // focused on tag↔name translation.
        let product = self.product_for(name);
        let len = rmp::decode::read_map_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack map length: {e:?}")))?;
        visitor.visit_map(TaggedProductMapAccess {
            parent: self,
            product,
            type_name: name,
            remaining: len as usize,
        })
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

impl<'a, 'de> Deserializer<'a, 'de> {
    /// Create a deserializer over some byte slice with a given registry.
    ///
    /// Uses the default configuration for the inner msgpack deserializer.
    fn new(bytes: &'de [u8], registry: &'a TagRegistry) -> Self {
        let inner = RmpDeserializer::new(bytes);
        Self { inner, registry }
    }

    /// Resolve a registered `Product` by serde name. Used by
    /// `deserialize_struct` (and, once it lands, `deserialize_tuple_struct`).
    /// Mirrors `Serializer::product_for` — a registry miss or sum-shaped
    /// entry is a real bug, so we panic loudly per the design doc rather
    /// than fabricating a synthetic shape.
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
pub(crate) struct TaggedAccessViaParent<'der, 'a, 'de> {
    parent: &'der mut Deserializer<'a, 'de>,
    remaining: usize,
}

/// Variable-length sequences and fixed-length tuples — both wire-encoded
/// as msgpack arrays. `next_element_seed` decrements `remaining` and
/// deserializes one element through the parent.
impl<'de, 'der, 'a> SeqAccess<'de> for TaggedAccessViaParent<'der, 'a, 'de> {
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
impl<'de, 'der, 'a> MapAccess<'de> for TaggedAccessViaParent<'der, 'a, 'de> {
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

/// `MapAccess` adapter for tagged structs (and tuple structs once that
/// lands). The wire is an int-keyed msgpack map — each entry's key is an
/// integer tag that we translate back to the registered field name before
/// handing it to the visitor (which expects a string identifier). Tags
/// not in `product.fields` are either silently skipped (`allow_unknown_tags
/// = true`) or rejected (the strict default).
///
/// Mirror of the serializer's `TaggedSerializeProduct` on the
/// `SerializeStruct` case — same `(int_tag, value)` map shape, just
/// driving the translation in the other direction.
pub(crate) struct TaggedProductMapAccess<'der, 'a, 'de> {
    parent: &'der mut Deserializer<'a, 'de>,
    product: crate::Product,
    /// Type name; only used for clearer error messages on unknown tags.
    type_name: &'static str,
    remaining: usize,
}

impl<'de, 'der, 'a> MapAccess<'de> for TaggedProductMapAccess<'der, 'a, 'de> {
    type Error = RmpError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        // Loop because unknown-but-skippable tags consume an entry from
        // the wire without yielding one to the visitor — we then continue
        // to the next entry. A `return Ok(None)` only happens when the
        // map is exhausted, and a `return Ok(Some(...))` only happens
        // when we have a known field name to hand to the visitor.
        loop {
            if self.remaining == 0 {
                return Ok(None);
            }
            self.remaining -= 1;
            let tag: u8 = u8::deserialize(&mut *self.parent)?;
            if let Some(field_name) = self.product.field_for(tag) {
                let key_deserializer =
                    de::value::BorrowedStrDeserializer::<RmpError>::new(field_name);
                return seed.deserialize(key_deserializer).map(Some);
            }
            // TODO: also auto-skip `self.product.is_reserved(tag)` here
            // — retired tags should decode silently regardless of
            // `allow_unknown_tags`. See the file-level TODO.
            if self.product.allow_unknown_tags {
                de::IgnoredAny::deserialize(&mut *self.parent)?;
                continue;
            }
            return Err(RmpError::custom(format!(
                "MsgpackTagged: unknown wire tag {tag} for product {:?} and \
                 `allow_unknown_tags` is false",
                self.type_name,
            )));
        }
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

/// `SeqAccess` adapter for top-level tuple structs (multi-element
/// `struct Pair(u32, bool)` shapes). Decoded by tag, not by wire order:
/// every wire entry is buffered as `(tag, value-bytes)` upfront, and on
/// each `next_element_seed` we look up the tag corresponding to the
/// current source position via `Product.tag_for("0")`, `tag_for("1")`,
/// … and re-deserialize the matching entry's bytes through a freshly
/// constructed sub-wrapper (so any nested tagged types still see this
/// wrapper's interception).
///
/// This is what makes the deserializer robust to wire-order changes on
/// the serializer side: if the serializer is fixed to emit tag-ascending
/// (matching the design doc) — or some other order — we don't need to
/// touch this code, because we route by tag.
///
/// **Buffering.** `value-bytes` is captured by snapshotting the inner
/// reader's `&[u8]` before and after each `IgnoredAny::deserialize` walk
/// (which advances exactly one msgpack value worth of bytes). Slicing
/// the diff out of the original buffer gives us the exact byte range
/// for that one value. The captured slice is `&'de [u8]` — same
/// underlying buffer, no copy.
///
/// **Inline capacity.** `SmallVec<[_; 4]>` covers the typical tuple-
/// struct arity (current ACIR/Brillig types use exclusively newtypes;
/// our test fixtures top out at 3-element tuple structs) without a
/// heap allocation. Anything larger transparently spills.
pub(crate) struct TaggedTupleStructAccess<'der, 'a, 'de> {
    parent: &'der mut Deserializer<'a, 'de>,
    product: crate::Product,
    /// Buffered `(tag, value-bytes)` pairs in wire arrival order. We look
    /// these up by tag in `next_element_seed`; insertion order doesn't
    /// matter.
    entries: SmallVec<[(u8, &'de [u8]); 4]>,
    next_position: usize,
}

impl<'de, 'der, 'a> SeqAccess<'de> for TaggedTupleStructAccess<'der, 'a, 'de> {
    type Error = RmpError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let position = self.next_position;
        self.next_position += 1;

        // Look up the wire tag for this source position. Wire-name strings
        // are positional (`"0"`, `"1"`, …) — the macro lifts these into
        // the registered `Product`'s `fields` slice. If the position is
        // beyond the type's arity, the product won't have it and we report
        // exhaustion to the visitor.
        let position_name = position.to_string();
        let Some(expected_tag) = self.product.tag_for(&position_name) else {
            return Ok(None);
        };

        // Find the wire entry carrying that tag. If absent, report None —
        // serde-derive's visitor fills `#[serde(default)]` slots from
        // `Default` and reports `invalid length` on a missing required
        // position.
        let Some(&(_, value_bytes)) = self.entries.iter().find(|(tag, _)| *tag == expected_tag)
        else {
            return Ok(None);
        };

        // Sub-wrapper over the value's bytes. Sharing the parent's
        // registry keeps nested tagged-type lookups consistent. The
        // sub-deserializer's reader state starts fresh at the value's
        // first byte, so its own `marker` buffer is empty as expected.
        let mut sub_deserializer = Deserializer::new(value_bytes, self.parent.registry);
        seed.deserialize(&mut sub_deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len().saturating_sub(self.next_position))
    }
}
