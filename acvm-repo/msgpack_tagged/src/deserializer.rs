//! Tagged-map msgpack deserializer that wraps [`rmp_serde::Deserializer`].
//!
//! Mirrors [`crate::serializer::Serializer`]: each aggregate shape that the
//! macro emits — named struct, multi-element tuple struct, sequence, tuple,
//! map, option, newtype struct — is intercepted to translate integer wire
//! tags back to the serde field/variant names the [`Visitor`] expects, via
//! the [`TagRegistry`].
//!
//! The public entry point is [`msgpack_tagged_deserialize`], which builds
//! the registry up front via `T::register_into` and runs the bytes through
//! the wrapper.
//!
//! ## Known gaps vs. the design doc / macro syntax
//!
//! The wrapper isn't final — the bits below are accepted by
//! `#[derive(MsgpackTagged)]` today but the deserializer doesn't model
//! them yet.
//!
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
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, Error as _, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use smallvec::SmallVec;

use crate::{MsgpackTagged, TagRegistry};

/// `rmp_serde`'s decode-side error type, re-exported for our `Deserializer`
/// impl. Matches the encode-side `RmpError` re-export in `serializer.rs`.
type RmpError = rmp_serde::decode::Error;

/// Buffered `(wire_tag, value-bytes)` pairs from an int-keyed map. Inline
/// capacity 4 covers the typical tuple-struct / tuple-variant arity without
/// a heap allocation; anything larger transparently spills.
type IntKeyedEntries<'de> = SmallVec<[(u8, &'de [u8]); 4]>;

/// Tagged-map msgpack deserializer.
///
/// Constructed internally by [`msgpack_tagged_deserialize`]. Exposed as `pub`
/// so it can be named from rustdoc; the only stable entry point is still the
/// free function. A builder for strategy customization will land later
/// (mirroring the serializer's plan).
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
pub struct Deserializer<'a, 'de> {
    inner: RmpDeserializer<ReadReader<&'de [u8]>>,
    registry: &'a TagRegistry,
}

impl<'a, 'de> Deserializer<'a, 'de> {
    /// Construct a deserializer over `bytes` borrowing the caller-built
    /// `registry`. Symmetric with [`crate::Serializer::new`].
    pub fn new(bytes: &'de [u8], registry: &'a TagRegistry) -> Self {
        Self { inner: RmpDeserializer::new(bytes), registry }
    }

    /// Read a msgpack map header (`fixmap` / `map16` / `map32`) and return
    /// its length as `usize`. Thin wrapper over `rmp::decode::read_map_len`
    /// that bakes in our error-formatting + the `u32 → usize` cast every
    /// caller wants.
    fn read_map_len(&mut self) -> Result<usize, RmpError> {
        rmp::decode::read_map_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack map length: {e:?}")))
            .map(|n| n as usize)
    }

    /// Read a msgpack array header (`fixarray` / `array16` / `array32`)
    /// and return its length as `usize`. Sibling of [`Self::read_map_len`].
    fn read_array_len(&mut self) -> Result<usize, RmpError> {
        rmp::decode::read_array_len(self.inner.get_mut())
            .map_err(|e| RmpError::custom(format!("failed to read msgpack array length: {e:?}")))
            .map(|n| n as usize)
    }
}

/// Build the tag registry from `T::register_into`, then deserialize a value
/// of type `T` from `bytes` through a [`Deserializer`].
///
/// All tagged types are expected to be encoded under `Format::MsgpackTagged`
/// (int-keyed maps for [`crate::EncodingStrategy::Tagged`], positional
/// arrays for [`crate::EncodingStrategy::Array`] — the decoder probes the
/// wire shape per struct). Other formats route through their own decoders
/// upstream via the `Format` byte (defined in the `acir` crate).
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
// translated back to serde field/variant names.
// ============================================================================

impl<'a, 'de> de::Deserializer<'de> for &mut Deserializer<'a, 'de> {
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
        let reader_state_before: &'de [u8] = self.inner.get_ref();
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
        // Lenient on purpose: consume whatever single msgpack value sits at
        // this position and hand `()` to the visitor. Plain serde requires
        // the wire to carry `nil` to satisfy a `()` field; our wrapper
        // relaxes that so a `()` field can act as a portable "skip this
        // position" placeholder.
        //
        // Use case: a stripped-down DTO that wants to ignore part of the
        // wire while keeping the surrounding tag layout intact. The
        // canonical example is `ProgramWithoutBrillig` in
        // `acvm-repo/acir/src/lib.rs` — same tag layout as `Program`,
        // but `unconstrained_functions: ()` discards the brillig payload
        // so the C++ codegen consumer can read just the ACIR section.
        // The C++ side achieves the same effect with `std::monostate`;
        // this hook keeps Rust-side decoding symmetric.
        //
        // Unit-variant payloads (encoded as a literal `nil`) flow through
        // here too and still work — `IgnoredAny` reads the nil byte and
        // we visit `()` exactly as before.
        de::IgnoredAny::deserialize(&mut *self)?;
        visitor.visit_unit()
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

    // -------- collection / aggregate shapes

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Read the msgpack array header (length-prefixed; `read_array_len`
        // consumes the marker and returns the element count). Unlike the
        // option case, the marker is metadata — the elements come AFTER
        // it — so consuming it is correct and we don't need to restore.
        // The adapter then yields each element via `&mut *self.parent`
        // so any tagged element inside the sequence recurses through
        // this wrapper.
        //
        // Byte-shaped values (`Vec<u8>`-equivalents like `FieldElement`)
        // arrive via `deserialize_bytes`, not `deserialize_seq` — see
        // the comment on `FieldElement::serialize` for why the encode
        // side uses `serialize_bytes` directly. So `deserialize_seq`
        // doesn't need to peek for `bin` markers here.
        let len = self.read_array_len()?;
        visitor.visit_seq(TaggedAccessViaParent { parent: self, remaining: len })
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
        let actual = self.read_array_len()?;
        if actual != len {
            return Err(RmpError::custom(format!(
                "tuple length mismatch: type expects {len} elements, wire has {actual}",
            )));
        }
        visitor.visit_seq(TaggedAccessViaParent { parent: self, remaining: len })
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        // Unused: per-position dispatch is driven by `TaggedTupleStructAccess`
        // (which knows the arity via the visitor itself). Tag-set validation
        // is in `buffer_and_validate_int_keyed_map`.
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // Top-level tuple structs (`struct Pair(u32, bool)`) come in two
        // wire shapes:
        //   * `Map` (Tagged): int-keyed `{tag → value}` pairs. Each
        //     entry's tag is read from the wire; we buffer
        //     `(tag, value-bytes)` and dispatch by tag in
        //     `next_element_seed` so reconstruction is robust to whatever
        //     order the serializer emits entries in.
        //   * `Array` (positional): bare `[v0, v1, …]`. Each entry's tag
        //     is synthesized from `product.fields[wire_pos]` (the wire is
        //     in tag-ascending order). The downstream
        //     `TaggedTupleStructAccess` dispatch is identical to the Map
        //     case once the buffer is materialized.
        let context = || format!("tuple-struct {name:?}");
        let product = self.product_for(name);
        let entries = match self.peek_wire_shape()? {
            WireShape::Map => {
                let wire_len = self.read_map_len()?;
                self.buffer_and_validate_int_keyed_map(wire_len, &product, context)?
            }
            WireShape::Array => {
                let wire_len = self.read_array_len()?;
                self.buffer_positional_array(wire_len, &product, context)?
            }
        };
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
        let len = self.read_map_len()?;
        visitor.visit_map(TaggedAccessViaParent { parent: self, remaining: len })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        // Look up the type's `Product` in the registry — it carries the
        // tag ↔ field-name mapping we need to feed the visitor — then peek
        // the wire shape to dispatch:
        //   * `Map` → existing `TaggedProductMapAccess`, reading tags from
        //     the wire and translating to field names.
        //   * `Array` → `ArrayProductMapAccess`, walking `product.fields`
        //     positionally and synthesizing the field-name keys.
        // In either case the visitor receives `(field_name, value)` pairs
        // via `visit_map` — that's how serde-derive's `Deserialize` for
        // named structs is driven — and serde-derive's `#[serde(default)]`
        // machinery covers any missing trailing positions on short wires.
        let product = self.product_for(name);
        match self.peek_wire_shape()? {
            WireShape::Map => {
                let len = self.read_map_len()?;
                visitor.visit_map(TaggedProductMapAccess {
                    parent: self,
                    product,
                    type_name: name,
                    remaining: len,
                })
            }
            WireShape::Array => {
                let wire_len = self.read_array_len()?;
                let layout = merged_wire_layout(&product);
                // Cap check: extras past the merged (active + reserved)
                // layout are only allowed when `allow_unknown_tags` is set.
                // Reserved slots inside the cap are drained silently inside
                // `ArrayProductMapAccess`.
                if !product.allow_unknown_tags && wire_len > layout.len() {
                    return Err(RmpError::custom(format!(
                        "struct {name:?}: wire has {wire_len} positional entries but the \
                         type accepts at most {} under Array strategy ({} active + {} \
                         reserved); opt into `#[tagged(allow_unknown_tags)]` to \
                         silently skip extras",
                        layout.len(),
                        product.fields.len(),
                        product.reserved.len(),
                    )));
                }
                visitor.visit_map(ArrayProductMapAccess {
                    parent: self,
                    product,
                    layout,
                    wire_remaining: wire_len,
                    next_position: 0,
                })
            }
        }
    }

    // Outer wire shape: a 1-entry msgpack map `{u8 variant_tag: payload}`,
    // mirroring `Serializer::write_variant_header`. We read the map header
    // and the variant tag here, then hand off to `TaggedEnumAccess` which
    // (a) yields the variant *name* to the visitor via
    // `BorrowedStrDeserializer` and (b) implements `VariantAccess` by
    // dispatching on the registered variant's `VariantKind`. Payload
    // values route back through `&mut self.parent` so nested tagged types
    // recurse through this wrapper.
    //
    // Lenient handling of unknown / reserved variant tags is not modeled
    // yet — see the file-level note about `allow_unknown` / `allow_reserved`.
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let sum = self.sum_for(name);
        let outer_len = self.read_map_len()?;
        if outer_len != 1 {
            return Err(RmpError::custom(format!(
                "MsgpackTagged: enum {name:?} expects a 1-entry outer map, got {outer_len}",
            )));
        }
        let tag: u8 = u8::deserialize(&mut *self)?;
        if let Some(variant) = sum.variants.iter().find(|v| v.tag == tag).copied() {
            return visitor.visit_enum(TaggedEnumAccess {
                parent: self,
                variant,
                payload_already_consumed: false,
            });
        }
        // Unknown wire tag. Strict per-marker routing:
        //   * tag in `sum.reserved` → only `on_reserved_tag` catches it
        //   * tag in neither variants nor reserved → only `on_unknown_tag`
        //   * otherwise → error
        // The two cases are intentionally non-overlapping: a user who wants
        // unified handling puts both `#[tagged(on_reserved)]` and
        // `#[tagged(on_unknown)]` on a single variant, in which case both
        // `Sum` fields point at the same tag and either path lands there.
        // A user who marks only one is making a deliberate "this kind of
        // drift is tolerable, that kind isn't" choice and we honor it.
        //
        // We drain the payload bytes before visiting so the outer stream
        // stays positioned after this enum value, then visit with
        // `payload_already_consumed: true` so `unit_variant` doesn't try to
        // re-read a `nil` that isn't there. The fallback is macro-validated
        // to be a unit variant.
        let fallback_tag =
            if sum.reserved.contains(&tag) { sum.on_reserved_tag } else { sum.on_unknown_tag };
        if let Some(fallback_tag) = fallback_tag {
            let fallback = sum
                .variants
                .iter()
                .find(|v| v.tag == fallback_tag)
                .copied()
                .expect("fallback tag must refer to a registered variant");
            de::IgnoredAny::deserialize(&mut *self)?;
            return visitor.visit_enum(TaggedEnumAccess {
                parent: self,
                variant: fallback,
                payload_already_consumed: true,
            });
        }
        Err(RmpError::custom(
            format!("MsgpackTagged: unknown variant tag {tag} for enum {name:?}",),
        ))
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

    /// Resolve a registered `Sum` by enum-type name. Mirror of `product_for`
    /// on the enum side, used by `deserialize_enum`.
    fn sum_for(&self, name: &'static str) -> crate::Sum {
        let entry = self.registry.get(name).unwrap_or_else(|| {
            panic!(
                "MsgpackTagged registry miss for enum {name:?} — the top-level \
                 `register_into` walk should have registered every reachable type"
            )
        });
        entry.tagged().as_sum().unwrap_or_else(|| {
            panic!("registry entry for {name:?} is product-shaped but a sum shape was expected")
        })
    }

    /// Read `wire_len` `(u8 tag, value-bytes)` pairs from the inner reader
    /// and return them as a `SmallVec` for later tag-driven dispatch. Used
    /// by both `deserialize_tuple_struct` and `TaggedEnumAccess::tuple_variant`
    /// — they share the int-keyed-map wire shape for tuple-shaped payloads
    /// but their visitors call back positionally rather than by name, so we
    /// can't dispatch entry-by-entry the way `TaggedProductMapAccess` does.
    ///
    /// Validates tag membership against the type's registered `Product`:
    /// * Tag in `product.fields` (active position) — fine.
    /// * Tag in `product.reserved` (retired position) — fine, the visitor
    ///   will simply never query it.
    /// * Otherwise — only fine if `product.allow_unknown_tags` is set.
    ///
    /// Performs a cheap upfront cap check (`wire_len > active + reserved`)
    /// when `allow_unknown_tags` is off, so grossly oversized wires error
    /// before we walk the bytes. The per-tag scan after buffering catches
    /// the within-bounds-but-still-unknown case.
    ///
    /// `context` is a caller-supplied closure producing a label (e.g.
    /// `"tuple-struct \"Foo\""` or `"tuple variant \"Carry\""`) that gets
    /// embedded in error messages. It's a closure so the `String` is only
    /// allocated on the error path — the happy path skips the format call.
    fn buffer_and_validate_int_keyed_map<'der>(
        &'der mut self,
        wire_len: usize,
        product: &crate::Product,
        context: impl FnOnce() -> String,
    ) -> Result<IntKeyedEntries<'de>, RmpError> {
        if !product.allow_unknown_tags {
            let cap = product.fields.len() + product.reserved.len();
            if wire_len > cap {
                return Err(RmpError::custom(format!(
                    "{}: wire has {wire_len} entries but the type accepts \
                     at most {cap} ({} active + {} reserved); opt into \
                     `#[tagged(allow_unknown_tags)]` to silently skip extras",
                    context(),
                    product.fields.len(),
                    product.reserved.len(),
                )));
            }
        }
        let mut entries: IntKeyedEntries<'de> = SmallVec::with_capacity(wire_len);
        for _ in 0..wire_len {
            let tag: u8 = u8::deserialize(&mut *self)?;
            let before: &'de [u8] = self.inner.get_mut();
            de::IgnoredAny::deserialize(&mut *self)?;
            let after: &'de [u8] = self.inner.get_mut();
            let consumed = before.len() - after.len();
            entries.push((tag, &before[..consumed]));
        }
        if !product.allow_unknown_tags {
            for (tag, _) in &entries {
                if product.field_for(*tag).is_none() && !product.is_reserved(*tag) {
                    return Err(RmpError::custom(format!(
                        "MsgpackTagged: unknown wire tag {tag} for {} \
                         and `allow_unknown_tags` is false",
                        context(),
                    )));
                }
            }
        }
        Ok(entries)
    }

    /// Read `wire_len` positional `value-bytes` entries from the inner
    /// reader (the `Array`-strategy wire shape — `fixarray` of values, no
    /// per-entry tag) and return them as an `IntKeyedEntries` with the tag
    /// for each entry *synthesized* from the merged-sorted (active +
    /// reserved) tag layout. This lets the downstream
    /// `TaggedTupleStructAccess` dispatch by tag uniformly across both
    /// wire strategies; for tuple structs / tuple variants the access
    /// reuses the same tag-driven path.
    ///
    /// **Reserved-tag handling under Array.** When a V2 schema retires a
    /// field with `#[tagged(reserved(N))]`, the V1 wire (produced before
    /// the retirement) still carries a value at the slot that used to
    /// hold tag `N`. The merged layout tells us which wire positions
    /// correspond to retired tags so we can drain them silently without
    /// confusing them with the active slots that come after.
    ///
    /// Cap check: `wire_len > active_count + reserved_count` is only
    /// accepted under `allow_unknown_tags`. Entries beyond that arity
    /// get consumed-and-discarded so the outer stream stays aligned, but
    /// they're not pushed to the returned buffer.
    fn buffer_positional_array<'der>(
        &'der mut self,
        wire_len: usize,
        product: &crate::Product,
        context: impl FnOnce() -> String,
    ) -> Result<IntKeyedEntries<'de>, RmpError> {
        let layout = merged_wire_layout(product);
        if !product.allow_unknown_tags && wire_len > layout.len() {
            return Err(RmpError::custom(format!(
                "{}: wire has {wire_len} positional entries but the type accepts \
                 at most {} under Array strategy ({} active + {} reserved); opt \
                 into `#[tagged(allow_unknown_tags)]` to silently skip extras",
                context(),
                layout.len(),
                product.fields.len(),
                product.reserved.len(),
            )));
        }
        let mut entries: IntKeyedEntries<'de> =
            SmallVec::with_capacity(wire_len.min(product.fields.len()));
        for wire_pos in 0..wire_len {
            let before: &'de [u8] = self.inner.get_mut();
            de::IgnoredAny::deserialize(&mut *self)?;
            let after: &'de [u8] = self.inner.get_mut();
            let consumed = before.len() - after.len();
            match layout.get(wire_pos) {
                // Active slot: push the value-bytes under the corresponding
                // tag so the downstream tag-driven access can find it.
                Some(&(tag, true)) => entries.push((tag, &before[..consumed])),
                // Reserved slot: V2 doesn't have an active field here; the
                // wire entry was V1's value at the retired position. Drain
                // silently — the value bytes are already consumed by
                // `IgnoredAny` above, and we don't push to the buffer.
                Some(&(_, false)) => {}
                // Trailing entry past `active + reserved`; cap check above
                // only lets us reach here under `allow_unknown_tags`.
                None => {}
            }
        }
        Ok(entries)
    }

    /// Peek the next msgpack marker without advancing the reader and
    /// classify it as a Tagged-strategy map header or an Array-strategy
    /// array header. Used at every product-shaped decode site
    /// (`deserialize_struct`, `deserialize_tuple_struct`,
    /// `TaggedEnumAccess::tuple_variant` / `struct_variant`) to dispatch
    /// to the right reader.
    ///
    /// Any other marker is a malformed-bytes error under
    /// `Format::MsgpackTagged` — legacy `Msgpack` / `MsgpackCompact` data
    /// has its own format byte and never reaches this wrapper, so
    /// string-keyed maps and other shapes are not expected here.
    fn peek_wire_shape(&mut self) -> Result<WireShape, RmpError> {
        let bytes: &'de [u8] = self.inner.get_mut();
        let first = *bytes.first().ok_or_else(|| {
            RmpError::custom("expected fixmap or fixarray under MsgpackTagged, got end-of-stream")
        })?;
        match Marker::from_u8(first) {
            Marker::FixMap(_) | Marker::Map16 | Marker::Map32 => Ok(WireShape::Map),
            Marker::FixArray(_) | Marker::Array16 | Marker::Array32 => Ok(WireShape::Array),
            m => Err(RmpError::custom(format!(
                "expected fixmap or fixarray under MsgpackTagged, got {m:?}"
            ))),
        }
    }
}

/// Wire shape of a product value under `Format::MsgpackTagged`. Picked per
/// struct at decode time by peeking the next msgpack marker — the
/// `Serializer` choice of `EncodingStrategy` is not communicated through a
/// separate channel (and doesn't need to be: the shape is self-describing
/// at the msgpack-header level).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WireShape {
    /// `fixmap` / `map16` / `map32` — Tagged: int-keyed map.
    Map,
    /// `fixarray` / `array16` / `array32` — Array: positional.
    Array,
}

/// Merge `product.fields` and `product.reserved` into a single
/// tag-ascending list of `(tag, is_active)` pairs.
///
/// The Array-strategy encoder emits wire positions in tag-ascending order
/// across both active and reserved tags (a V1 writer that still carries a
/// value at a now-retired tag sorts that tag in alongside the live ones).
/// So decoding by wire position requires the same merged ordering: position
/// `i` on the wire corresponds to the i-th tag in the merged list, and the
/// `is_active` flag tells the decoder whether to push the value bytes into
/// the buffered entries (active) or drain them silently (reserved).
///
/// Inline capacity 8 fits the typical ACIR product without spilling; larger
/// products transparently fall back to the heap.
fn merged_wire_layout(product: &crate::Product) -> SmallVec<[(u8, bool); 8]> {
    let mut layout: SmallVec<[(u8, bool); 8]> =
        SmallVec::with_capacity(product.fields.len() + product.reserved.len());
    for &(tag, _) in product.fields {
        layout.push((tag, true));
    }
    for &tag in product.reserved {
        layout.push((tag, false));
    }
    layout.sort_by_key(|(tag, _)| *tag);
    layout
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
struct TaggedAccessViaParent<'der, 'a, 'de> {
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
struct TaggedProductMapAccess<'der, 'a, 'de> {
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
            // Tag isn't an active field. Two ways to tolerate it:
            //   * `product.is_reserved(tag)` → the type explicitly retired
            //     this tag, so the user has opted into silent-skip for it
            //     (parallel to enums' `on_reserved` marker). Always silent.
            //   * `product.allow_unknown_tags` → the type opted into blanket
            //     forward-compat for *any* tag it doesn't recognize.
            // Either branch consumes the value and loops to the next entry.
            if self.product.is_reserved(tag) || self.product.allow_unknown_tags {
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

/// `MapAccess` adapter for named structs decoded from the **Array**
/// strategy wire shape (positional `fixarray` of values). Unlike
/// `TaggedProductMapAccess` the wire carries no tags; we synthesize the
/// keys yielded to the visitor by walking the merged (active + reserved)
/// wire layout — see [`merged_wire_layout`] for why the merge is needed.
/// The visitor still receives `(field_name, value)` pairs — that's how
/// serde-derive's `Deserialize` for named structs is driven — but the
/// field-name source is the registry, not the wire.
///
/// Reserved slots inside the layout are drained transparently: the wire
/// position is consumed (so the outer stream stays aligned), but no
/// `(field_name, value)` pair is yielded to the visitor.
///
/// Short wires (`wire_remaining < active count`) yield `Ok(None)` once
/// exhausted; serde-derive's `#[serde(default)]` machinery fills in any
/// missing trailing positions, same fallback as the Tagged path.
///
/// Long wires (`wire_remaining > layout.len()`) only reach this adapter
/// when `allow_unknown_tags` is set — the cap check at the call site
/// bails on the strict default. Extras past the layout get consumed-and-
/// discarded inside `next_key_seed`'s loop before reporting exhaustion.
struct ArrayProductMapAccess<'der, 'a, 'de> {
    parent: &'der mut Deserializer<'a, 'de>,
    product: crate::Product,
    /// Merged (active + reserved) wire layout, tag-ascending. Drives the
    /// per-position dispatch between "yield to visitor" (active) and
    /// "drain silently" (reserved).
    layout: SmallVec<[(u8, bool); 8]>,
    /// Wire entries left to consume.
    wire_remaining: usize,
    /// Next position in the merged layout to consider.
    next_position: usize,
}

impl<'de, 'der, 'a> MapAccess<'de> for ArrayProductMapAccess<'der, 'a, 'de> {
    type Error = RmpError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        loop {
            // Wire exhausted: serde-derive's `#[serde(default)]` covers
            // any trailing active positions the type still expects.
            if self.wire_remaining == 0 {
                return Ok(None);
            }
            match self.layout.get(self.next_position) {
                // Reserved slot: discard the value bytes and advance both
                // counters, then look at the next position.
                Some(&(_, false)) => {
                    de::IgnoredAny::deserialize(&mut *self.parent)?;
                    self.wire_remaining -= 1;
                    self.next_position += 1;
                }
                // Active slot: synthesize the field-name key and hand it
                // to the visitor. The matching `next_value_seed` call
                // below advances the counters.
                Some(&(tag, true)) => {
                    let field_name = self
                        .product
                        .field_for(tag)
                        .expect("active layout entry must resolve to a field name");
                    let key_deserializer =
                        de::value::BorrowedStrDeserializer::<RmpError>::new(field_name);
                    return seed.deserialize(key_deserializer).map(Some);
                }
                // Past the merged layout (only reachable under
                // `allow_unknown_tags`). Drain remaining wire entries
                // silently — never yield to the visitor.
                None => {
                    de::IgnoredAny::deserialize(&mut *self.parent)?;
                    self.wire_remaining -= 1;
                }
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.parent)?;
        self.wire_remaining -= 1;
        self.next_position += 1;
        Ok(value)
    }

    fn size_hint(&self) -> Option<usize> {
        // Upper bound: active positions still pending in the layout, capped
        // by how many wire entries remain.
        let active_remaining = self.layout[self.next_position.min(self.layout.len())..]
            .iter()
            .filter(|(_, active)| *active)
            .count();
        Some(active_remaining.min(self.wire_remaining))
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
struct TaggedTupleStructAccess<'der, 'a, 'de> {
    parent: &'der mut Deserializer<'a, 'de>,
    product: crate::Product,
    /// Buffered `(tag, value-bytes)` pairs in wire arrival order. We look
    /// these up by tag in `next_element_seed`; insertion order doesn't
    /// matter.
    entries: IntKeyedEntries<'de>,
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
        // registry (via `Arc::clone`) keeps nested tagged-type lookups
        // consistent. The sub-deserializer's reader state starts fresh at
        // the value's first byte, so its own `marker` buffer is empty as
        // expected.
        let mut sub_deserializer = Deserializer::new(value_bytes, self.parent.registry);
        seed.deserialize(&mut sub_deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len().saturating_sub(self.next_position))
    }
}

/// `EnumAccess` + `VariantAccess` adapter for tagged enums.
///
/// `deserialize_enum` already consumed the outer `{u8 variant_tag: payload}`
/// map header and the variant tag, resolved the matching `Variant` from the
/// `Sum`, and handed both off to us. From here:
///
/// * `EnumAccess::variant_seed` yields the variant's serde *name* (as a
///   borrowed string) to the visitor — serde-derive's `__Field` visitor for
///   enums accepts identifiers as strings just like struct-field visitors do.
/// * The four `VariantAccess` methods dispatch on the kind the visitor
///   asks for (which is driven by the Rust declaration, not the wire), and
///   each reads the matching payload shape directly from `parent`:
///   - **unit** — consume the trailing `nil` written by the encode-side
///     `serialize_unit`.
///   - **newtype** — pass the bare inner value through `&mut *parent` so
///     any nested tagged types still recurse through the wrapper.
///   - **tuple** — read the int-keyed payload map and reuse the
///     `TaggedTupleStructAccess` buffering machinery; the variant's
///     `payload` `Product` carries the tag/position mapping.
///   - **struct** — read the int-keyed payload map and reuse
///     `TaggedProductMapAccess`; the variant's `payload` `Product` carries
///     the tag/field-name mapping.
struct TaggedEnumAccess<'der, 'a, 'de> {
    parent: &'der mut Deserializer<'a, 'de>,
    variant: crate::Variant,
    /// Set when `deserialize_enum` has already drained the wire payload —
    /// the only way to land here is the catch-all route, where the
    /// payload was discarded with `IgnoredAny` before `visit_enum`. The
    /// catch-all is always a unit variant per macro validation, so only
    /// `unit_variant` needs to consult this flag.
    payload_already_consumed: bool,
}

impl<'de, 'der, 'a> EnumAccess<'de> for TaggedEnumAccess<'der, 'a, 'de> {
    type Error = RmpError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let key_deserializer =
            de::value::BorrowedStrDeserializer::<RmpError>::new(self.variant.name);
        let value = seed.deserialize(key_deserializer)?;
        Ok((value, self))
    }
}

impl<'de, 'der, 'a> VariantAccess<'de> for TaggedEnumAccess<'der, 'a, 'de> {
    type Error = RmpError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        if self.payload_already_consumed {
            return Ok(());
        }
        <()>::deserialize(&mut *self.parent)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.parent)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Same dispatch as the top-level tuple-struct path: peek the
        // payload's wire shape, buffer into `IntKeyedEntries` either by
        // reading wire tags (Tagged) or by synthesizing them from the
        // payload's `Product.fields` (Array), then hand off to the
        // shared `TaggedTupleStructAccess`.
        let variant_name = self.variant.name;
        let context = || format!("tuple variant {variant_name:?}");
        let entries = match self.parent.peek_wire_shape()? {
            WireShape::Map => {
                let wire_len = self.parent.read_map_len()?;
                self.parent.buffer_and_validate_int_keyed_map(
                    wire_len,
                    &self.variant.payload,
                    context,
                )?
            }
            WireShape::Array => {
                let wire_len = self.parent.read_array_len()?;
                self.parent.buffer_positional_array(wire_len, &self.variant.payload, context)?
            }
        };
        visitor.visit_seq(TaggedTupleStructAccess {
            parent: self.parent,
            product: self.variant.payload,
            entries,
            next_position: 0,
        })
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Mirror of `Deserializer::deserialize_struct`'s dispatch, on the
        // variant payload's `Product`.
        match self.parent.peek_wire_shape()? {
            WireShape::Map => {
                let wire_len = self.parent.read_map_len()?;
                visitor.visit_map(TaggedProductMapAccess {
                    parent: self.parent,
                    product: self.variant.payload,
                    type_name: self.variant.name,
                    remaining: wire_len,
                })
            }
            WireShape::Array => {
                let wire_len = self.parent.read_array_len()?;
                let layout = merged_wire_layout(&self.variant.payload);
                if !self.variant.payload.allow_unknown_tags && wire_len > layout.len() {
                    return Err(RmpError::custom(format!(
                        "struct variant {:?}: wire has {wire_len} positional entries but the \
                         payload accepts at most {} under Array strategy ({} active + {} \
                         reserved); opt into `#[tagged(allow_unknown_tags)]` to silently \
                         skip extras",
                        self.variant.name,
                        layout.len(),
                        self.variant.payload.fields.len(),
                        self.variant.payload.reserved.len(),
                    )));
                }
                visitor.visit_map(ArrayProductMapAccess {
                    parent: self.parent,
                    product: self.variant.payload,
                    layout,
                    wire_remaining: wire_len,
                    next_position: 0,
                })
            }
        }
    }
}
