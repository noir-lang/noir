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
//!
//! ## Known gaps vs. the design doc / macro syntax
//!
//! The wrapper isn't final — the bits below are accepted by
//! `#[derive(MsgpackTagged)]` today but aren't reflected in the wire bytes
//! we produce yet.
//!
//! - **Tag-ascending wire order.** The design promises field/element
//!   entries on the wire in tag-ascending order so two semantically-equal
//!   values encode byte-identically regardless of source-declaration
//!   order. We currently emit in serde's call-order = source-declaration
//!   order. Tightening this requires buffering field bytes before writing.
//! - **Encoding strategies.** Only the **Tagged** strategy (int-keyed
//!   map) is implemented. Per-type strategy overrides — **Array**
//!   (positional msgpack array, smallest wire) and **Named** (rmp_serde
//!   default, string-keyed map) — are deferred follow-ups.
//! - **`assert_eq!` on `len` vs `product.fields.len()`** — already
//!   tightened, but only inside the four product-shaped methods. New
//!   shapes that join the family should add the same assert.

use std::collections::HashMap;
use std::io::Write;

use rmp_serde::Serializer as RmpSerializer;
// `ser::Serializer` would clash with our own `Serializer` struct below if
// pulled in via `use`; importing the `ser` module instead lets us write
// `ser::Serializer` for the trait at the few sites that need it.
use serde::ser::{
    self, Error as _, Serialize, SerializeMap, SerializeSeq, SerializeStruct,
    SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::{EncodingStrategy, MsgpackTagged, TagRegistry, type_name_basename};

/// Tagged-map msgpack serializer.
///
/// Borrows a caller-owned [`TagRegistry`] and carries per-encode-session
/// policy (default strategy + per-type overrides). The registry stays pure
/// type metadata — strategy state lives only on the serializer. Typical
/// usage:
///
/// ```ignore
/// let registry = TagRegistry::from_type::<Program<F>>();
/// let mut buf = Vec::new();
/// let mut s = msgpack_tagged::Serializer::new(&mut buf, &registry)
///     .with_default_strategy(EncodingStrategy::Array)
///     .with_strategy::<Program<F>>(EncodingStrategy::Tagged)
///     .with_strategy::<Circuit<F>>(EncodingStrategy::Tagged);
/// program.serialize(&mut s)?;
/// ```
///
/// `new` defaults the strategy to [`EncodingStrategy::Tagged`] (the most
/// evolution-friendly shape); `with_default_strategy` swaps it, and
/// `with_strategy::<U>` overrides for individual types (panicking on a
/// registry miss). Policy sites that target types by string name —
/// possibly without knowing whether the type is reachable — use
/// `with_strategy_for_name` instead.
pub struct Serializer<'a, W: Write> {
    inner: RmpSerializer<W>,
    registry: &'a TagRegistry,
    default_strategy: EncodingStrategy,
    /// Per-type strategy overrides keyed by **serde name** (the name
    /// `#[serde(rename = "...")]` resolves to, or the bare type ident when
    /// no rename is set). Keying by name — not [`std::any::TypeId`] — lets
    /// `with_strategy::<Foo<F>>` apply uniformly across field flavors
    /// (`Foo<FieldElement>` and `Foo<OtherF>` share the name "Foo") and
    /// across shadow DTOs (a public `Circuit<F>` with
    /// `#[tagged(via(CircuitWire<F>))]` reaches the same "Circuit"
    /// override that CircuitWire registers under via `#[serde(rename)]`).
    /// See [`type_name_basename`] for the derivation rule.
    overrides: HashMap<&'static str, EncodingStrategy>,
}

impl<'a, W: Write> Serializer<'a, W> {
    /// Construct a serializer over `writer` borrowing the caller-built
    /// `registry`. The default per-type strategy is
    /// [`EncodingStrategy::Tagged`].
    ///
    /// Build the registry up front via `TagRegistry::from_type::<T>()` for
    /// the top-level type being serialized — the registration walk seeds
    /// every nested tagged type.
    pub fn new(writer: W, registry: &'a TagRegistry) -> Self {
        // Tagged types' `serialize_struct` / variant calls route
        // through our interception layer below; the inner
        // `RmpSerializer` only gets the structurally-irrelevant calls
        // (primitives, `serialize_bytes`, etc.). See
        // [`make_inner_rmp_serializer`] for why no `BytesMode`
        // override is applied (and the rule for new bytes-shaped
        // types: call `serialize_bytes` directly in their
        // `Serialize` impl).
        Self {
            inner: make_inner_rmp_serializer(writer),
            registry,
            default_strategy: EncodingStrategy::Tagged,
            overrides: HashMap::new(),
        }
    }

    /// Change the default strategy applied to types without a per-type
    /// override.
    pub fn with_default_strategy(mut self, strategy: EncodingStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    /// Set the encoding strategy for a specific type `T`. Overrides the
    /// default for this type only; later calls for the same `T` win.
    /// Resolves `T`'s serde name via [`type_name_basename`] and inserts
    /// the override under that name.
    ///
    /// **Panics** if `T`'s name isn't in the registry — setting a
    /// strategy for an unreachable type is almost always a bug (the
    /// override would silently never fire). Use this at call sites where
    /// the override targets a type the caller *knows* should be
    /// reachable (typically tests, or a producer that just registered
    /// the type up front). Policy sites that configure overrides for a
    /// fixed catalog of potentially-top-level types — where the caller's
    /// `T` might not reach all of them — should use
    /// [`Self::with_strategy_for_name`] instead.
    pub fn with_strategy<T: MsgpackTagged>(mut self, strategy: EncodingStrategy) -> Self {
        let name = type_name_basename::<T>();
        assert!(
            self.registry.contains(name),
            "Serializer::with_strategy: serde name {name:?} (from type {full_name}) is \
             not in the registry — the top-level type's `register_into` walk didn't \
             reach it. Build the registry from a type that transitively visits T, use \
             `with_strategy_for_name` for policy sites that tolerate unreachable names, \
             or remove the override.",
            full_name = std::any::type_name::<T>(),
        );
        self.overrides.insert(name, strategy);
        self
    }

    /// Set the encoding strategy for the type registered under serde
    /// `name`. Never asserts — names not in the registry get a stray
    /// override entry that's never looked up at encode time (harmless).
    /// Sibling of [`Self::with_strategy`] for *policy* sites that
    /// configure overrides for a fixed catalog of potentially-top-level
    /// types but don't know which ones the caller will actually
    /// serialize.
    ///
    /// For tests or producer call sites that want to fail fast on a
    /// registry miss, use [`Self::with_strategy`] instead.
    pub fn with_strategy_for_name(
        mut self,
        name: &'static str,
        strategy: EncodingStrategy,
    ) -> Self {
        self.overrides.insert(name, strategy);
        self
    }

    /// Look up the effective encoding strategy for the registered type
    /// `T`. Returns the per-type override if one was set; otherwise the
    /// default strategy. Used by the encode-time dispatch and exposed for
    /// tests.
    pub fn strategy_for<T: MsgpackTagged>(&self) -> EncodingStrategy {
        self.strategy_for_name(type_name_basename::<T>())
    }
}

/// Build the tag registry from `T::register_into`, then serialize `value`
/// through a [`Serializer`] into a freshly-allocated `Vec<u8>`. Uses the
/// default [`EncodingStrategy::Tagged`] for all types. For strategy
/// customization, build the registry and serializer directly and use the
/// builder methods.
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

impl<'ser, 'a, W: Write> ser::Serializer for &'ser mut Serializer<'a, W> {
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
        // Same on-wire shape as a named struct — the registered `Product`
        // uses positional wire-names ("0", "1", …) instead of field idents.
        // Both shapes share the same adapter; the trait impl that fires is
        // chosen by serde based on which `serialize_*` call landed.
        self.begin_product(name, len)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.begin_product(name, len)
    }

    // -------- sum shapes: every variant becomes `{<variant_tag>: <payload>}`,
    // with the payload shape determined by `VariantKind`. Each branch looks
    // up the `Sum`, resolves the variant by name, writes a 1-entry outer map
    // header (variant tag → payload), then emits the payload appropriate to
    // the kind: `nil` for unit, the inner value pass-through for newtype, or
    // a `Product`-shaped payload for tuple/struct (whose shape follows the
    // enclosing enum's strategy).

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
        self.begin_variant_payload(name, variant, len)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.begin_variant_payload(name, variant, len)
    }
}

impl<'a, W: Write> Serializer<'a, W> {
    /// Shared body of `serialize_struct` / `serialize_tuple_struct`.
    /// Resolves the product + strategy, writes the strategy-appropriate
    /// resolves the product + strategy, returns a configured
    /// `TaggedSerializeProduct`. The two trait methods are identical at
    /// this level — only the trait impl fired on the returned adapter
    /// differs (which serde picks based on the caller's `serialize_*`
    /// choice).
    ///
    /// **Buffering policy.** Per-field bytes are buffered into the
    /// adapter and flushed in tag-ascending order at `end()` whenever
    /// the user's source-declaration order has been deliberately
    /// reordered relative to the tags
    /// (`!product.tag_order_matches_source`). This is what makes both
    /// strategies emit canonical (tag-ascending) wire order:
    ///
    /// * Under `Array` it's a *correctness* requirement — the decoder
    ///   reads positionally and would otherwise see fields in the wrong
    ///   slots.
    /// * Under `Tagged` it's a *byte-determinism* requirement — the
    ///   decoder doesn't care about order (every wire entry carries its
    ///   tag), but consumers reading the bytes do: cross-implementation
    ///   compatibility, hashing, cryptographic commitments. The design
    ///   doc's "TAGS define the canonical field order" promise applies
    ///   here.
    ///
    /// Types whose source-declaration order is already tag-ascending
    /// (the common case — newly-added types and types using implicit
    /// positional tags) skip the buffer entirely: the outer header is
    /// written upfront and each field streams through to the parent
    /// directly, saving the per-field `Vec<u8>` allocation. The cost is
    /// paid only by types whose tags have drifted out of source order
    /// — typically schema-evolved types with retired-and-re-added
    /// fields. **If you reorder fields, you're opting into a per-field
    /// allocation at encode time.**
    fn begin_product<'ser>(
        &'ser mut self,
        name: &'static str,
        len: usize,
    ) -> Result<TaggedSerializeProduct<'ser, 'a, W>, RmpError> {
        let (product, strategy) = self.product_and_strategy_for(name);
        assert_field_count_matches(name, len, product.fields.len());
        let strategy = downgrade_array_if_unsafe(&product, strategy);
        begin_product_payload(self, product, strategy)
    }

    /// Shared body of `serialize_tuple_variant` / `serialize_struct_variant`.
    /// Writes the outer 1-entry `{variant_tag: ...}` discriminator map
    /// (always Tagged — variant identification is by integer tag under
    /// `MsgpackTagged`); same buffering policy as `begin_product` for the
    /// payload itself.
    fn begin_variant_payload<'ser>(
        &'ser mut self,
        name: &'static str,
        variant: &'static str,
        len: usize,
    ) -> Result<TaggedSerializeProduct<'ser, 'a, W>, RmpError> {
        let v = self.write_variant_header(name, variant)?;
        assert_field_count_matches(variant, len, v.payload.fields.len());
        let strategy = self.strategy_for_name(name);
        let strategy = downgrade_array_if_unsafe(&v.payload, strategy);
        begin_product_payload(self, v.payload, strategy)
    }

    /// Spawn a sub-serializer over a fresh `Vec<u8>` buffer, inheriting
    /// this serializer's registry + per-type strategy config. Used by
    /// [`TaggedSerializeProduct::serialize_tag_and_value`] to encode each
    /// field's bytes into a temp buffer before flushing in tag-ascending
    /// order at `end()`. Cloning the `overrides` map is cheap in practice
    /// — it's tiny (a few entries) and only walked per top-level value's
    /// field count.
    fn sub_serializer_into<'sub>(
        &self,
        writer: &'sub mut Vec<u8>,
    ) -> Serializer<'a, &'sub mut Vec<u8>> {
        Serializer {
            inner: make_inner_rmp_serializer(writer),
            registry: self.registry,
            default_strategy: self.default_strategy,
            overrides: self.overrides.clone(),
        }
    }

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

    /// Resolve a registered `Product` *and* the effective encoding strategy
    /// for the type registered under `name`. Used only on the top-level
    /// struct paths — variant payloads are forced Tagged at their
    /// construction sites.
    fn product_and_strategy_for(&self, name: &'static str) -> (crate::Product, EncodingStrategy) {
        (self.product_for(name), self.strategy_for_name(name))
    }

    /// Resolve the effective encoding strategy for the type registered
    /// under serde `name`. Overrides are keyed by the same serde-name
    /// string the caller passes here, so it's a single hash lookup with
    /// no registry indirection. Absent any override the per-serializer
    /// `default_strategy` applies.
    fn strategy_for_name(&self, name: &str) -> EncodingStrategy {
        self.overrides.get(name).copied().unwrap_or(self.default_strategy)
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

/// Build the inner `rmp_serde::Serializer` for primitive / forwarded
/// calls. Single construction point so every spawn site
/// ([`Serializer::new`], [`Serializer::sub_serializer_into`]) stays in
/// lockstep.
///
/// **Why we don't apply `BytesMode::ForceIterables` here** (despite
/// the legacy `acir::serialization::msgpack_serialize` doing so):
///
/// `ForceIterables` makes `rmp_serde::Serializer::collect_seq` detect
/// byte-shaped iterators and emit msgpack `bin` instead of a
/// `fixarray` of `fixint`s — the wire shape the C++ codegen's
/// `std::vector<uint8_t>` adapter expects. But for that detection to
/// apply, the call has to *reach* the inner's `collect_seq`. Our
/// wrapper intercepts `Vec<T>::serialize` via the default
/// `collect_seq` → `Self::serialize_seq` path: `serialize_seq` writes
/// a `fixarray` header directly and routes each element through this
/// wrapper, so the inner's `ForceIterables` is never consulted.
///
/// We could override `collect_seq` to forward byte-shaped iterators
/// to `inner.collect_seq` — but rmp_serde's detection heuristic is
/// purely size-based: any iterator over pointer-sized items (`&u8`,
/// `Box<T>`, `Rc<T>`, `&T`, …) matches. For items that *aren't*
/// actually `u8`, rmp_serde's `OnlyBytes` probe rejects, and rmp_serde
/// falls back to its **own** `serialize_seq` — which doesn't route
/// through our wrapper. That would silently bypass `MsgpackTagged`
/// interception for any tagged type wrapped in `Box`/`&`/etc. inside
/// a `Vec`. Hard to debug, easy to land in.
///
/// Instead we keep the wrapper simple and require the load-bearing
/// case (`FieldElement`'s `Serialize` impl) to call
/// `serializer.serialize_bytes(...)` directly — bypassing
/// `collect_seq` entirely. `serialize_bytes` is `rmp_serde`'s
/// unconditional `write_bin` (independent of `BytesMode`), and our
/// own `serialize_bytes` forwards it untouched, so the wire is
/// reliably `bin` for that field.
///
/// **If a future model adds another byte-shaped value type**, prefer
/// hooking it up via `serialize_bytes` for the same reason. Only if a
/// generic byte-iter intercept becomes truly necessary should we
/// override `collect_seq` here — and at that point we'd need to
/// **also replicate rmp_serde's `OnlyBytes` probe** so the
/// reference-bypass risk above is closed.
fn make_inner_rmp_serializer<W: Write>(writer: W) -> RmpSerializer<W> {
    RmpSerializer::new(writer)
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

/// Auto-downgrade `Array` → `Tagged` for products where the positional
/// shape can't safely round-trip the type's own writes.
///
/// **The hazard.** Under `Array` the encoder only writes active fields
/// (in tag-ascending order); the decoder walks a merged-sorted layout of
/// `(active + reserved)` tags so it can drain reserved slots from
/// *legacy* wires (V1 wrote a value at a tag that V2 has since retired).
/// That works fine when reserved tags are all *strictly greater* than
/// every active tag: the merged layout puts them at the tail, and the
/// decoder hits `wire_remaining == 0` before visiting them. But if any
/// reserved tag has an active tag *after* it in tag order, the merged
/// layout interleaves a reserved slot in the middle. A round-trip of the
/// type's own write would then drain a wire byte the encoder intended
/// for the next active field, corrupting the decode.
///
/// **The fix.** When that interleaving is possible, the strategy
/// silently flips to `Tagged` for this product — the int-keyed-map shape
/// is self-describing on the wire (each entry carries its own tag) so
/// the decoder doesn't need positional alignment. The wire is slightly
/// larger but the type is now round-trip-safe.
///
/// **Why a silent downgrade rather than an error.** A bulk
/// `with_default_strategy(Array)` is the common config — flipping it to
/// an error per type would force callers to add a `with_strategy::<T>(Tagged)`
/// override for every schema-evolved leaf, which is noise. The downgrade
/// is local to the call and doesn't affect other types in the same
/// serializer. Documented in the crate README under the migration guide.
fn downgrade_array_if_unsafe(
    product: &crate::Product,
    strategy: EncodingStrategy,
) -> EncodingStrategy {
    if strategy != EncodingStrategy::Array || product.reserved.is_empty() {
        return strategy;
    }
    let max_active = product.fields.iter().map(|(t, _)| *t).max();
    let min_reserved = product.reserved.iter().copied().min();
    match (max_active, min_reserved) {
        // Some reserved tag falls at or before some active tag — the
        // unsafe interleaving case. Downgrade.
        (Some(active), Some(reserved)) if reserved <= active => EncodingStrategy::Tagged,
        // Either no active fields (the product is empty so nothing
        // round-trips through Array anyway) or reserved is strictly
        // trailing — Array is safe.
        _ => strategy,
    }
}

/// Write the outer Product header in the shape required by `strategy`:
/// `Tagged` → int-keyed `fixmap`, `Array` → positional `fixarray`. Used
/// by both top-level struct paths and variant-payload paths.
fn write_strategy_header<W: Write>(
    writer: &mut W,
    len: usize,
    strategy: EncodingStrategy,
) -> Result<(), RmpError> {
    match strategy {
        EncodingStrategy::Tagged => write_map_header(writer, len),
        EncodingStrategy::Array => write_array_header(writer, len),
    }
}

/// Assert that serde's reported field count matches the registered
/// `Product`'s — both should drop the same skipped fields, so they should
/// always agree. A mismatch is a real misconfiguration: the most common
/// cause is a `PhantomData<T>` field that's missing `#[serde(skip)]`,
/// which would otherwise fail later with a confusing "field not found in
/// registered Product" error from a tag lookup. The assert surfaces it
/// earlier, with a more actionable message.
fn assert_field_count_matches(name: &str, serde_len: usize, product_len: usize) {
    assert_eq!(
        serde_len, product_len,
        "MsgpackTagged: serde reports {serde_len} fields for {name:?} but the registered \
         Product carries {product_len}. The macro and `serde::Serialize` disagree on \
         which fields are on the wire — typically because a `PhantomData<T>` field \
         is missing `#[serde(skip)]`",
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
///
/// `strategy` drives the per-field emission: under `Tagged` each entry is
/// `(tag, value)` on the wire, under `Array` each entry is just `value`.
///
/// Behavior splits on the `buffer` flag, decided at `begin_product` time:
///
/// * `buffer = false` (the common path): the outer header is written
///   upfront and each `serialize_field` writes through the parent stream
///   immediately. Used when reordering can't change the wire (`Tagged`
///   with any source order, or `Array` whose source order is already
///   tag-ascending). The `entries` field stays empty.
/// * `buffer = true`: the outer header is *deferred* and `serialize_field`
///   encodes the value into a fresh `Vec<u8>` through a sub-serializer
///   that shares the parent's registry + strategy, then pushes
///   `(tag, bytes)` to `entries`. The `finish` flush at `end()` sorts by
///   tag and dumps in canonical order to the parent. Only reached for
///   `Array`-strategy types whose source-declaration order doesn't match
///   tag-ascending order.
pub struct TaggedSerializeProduct<'ser, 'a, W: Write> {
    product: crate::Product,
    parent: &'ser mut Serializer<'a, W>,
    next_position: usize,
    strategy: EncodingStrategy,
    buffer: bool,
    entries: Vec<(u8, Vec<u8>)>,
}

/// Decide whether to buffer + flush in tag-ascending order, write the
/// outer header upfront in the direct case, and return the configured
/// adapter. Shared by [`Serializer::begin_product`] (top-level) and
/// [`Serializer::begin_variant_payload`] (enum payload).
fn begin_product_payload<'ser, 'a, W: Write>(
    parent: &'ser mut Serializer<'a, W>,
    product: crate::Product,
    strategy: EncodingStrategy,
) -> Result<TaggedSerializeProduct<'ser, 'a, W>, RmpError> {
    // Buffer iff serde's call order won't naturally produce
    // tag-ascending output — i.e. whenever the user's source-declaration
    // order has been deliberately reordered relative to the tags. Both
    // strategies benefit from canonical wire order (Array for correctness,
    // Tagged for byte-determinism — cross-implementation compat, hashing
    // / commitment use cases). Types whose source order *is* monotonic
    // (the vast majority) pay no allocation regardless of strategy.
    let buffer = !product.tag_order_matches_source;
    if !buffer {
        write_strategy_header(parent.inner.get_mut(), product.fields.len(), strategy)?;
    }
    Ok(TaggedSerializeProduct {
        // Capacity matters only on the buffered path; on the direct path
        // entries stays empty and the heap allocation is skipped.
        entries: if buffer { Vec::with_capacity(product.fields.len()) } else { Vec::new() },
        product,
        parent,
        next_position: 0,
        strategy,
        buffer,
    })
}

impl<'ser, 'a, W: Write> TaggedSerializeProduct<'ser, 'a, W> {
    /// Emit one field's wire contribution. Direct path (the common case):
    /// write `(tag, value)` under Tagged or just `value` under Array,
    /// straight to the parent stream. Buffered path (Array with
    /// non-monotonic source order): encode the value into a temp
    /// `Vec<u8>` through a sub-serializer that shares the parent's
    /// registry + strategy and push `(tag, bytes)` to `entries` for
    /// later flushing in `finish`. Either path keeps nested tagged
    /// types recursing through the wrapper.
    fn serialize_tag_and_value<T>(&mut self, tag: u8, value: &T) -> Result<(), RmpError>
    where
        T: ?Sized + Serialize,
    {
        if self.buffer {
            let mut buf = Vec::new();
            {
                let mut sub = self.parent.sub_serializer_into(&mut buf);
                value.serialize(&mut sub)?;
            }
            self.entries.push((tag, buf));
        } else {
            if matches!(self.strategy, EncodingStrategy::Tagged) {
                ser::Serializer::serialize_u8(&mut *self.parent, tag)?;
            }
            value.serialize(&mut *self.parent)?;
        }
        Ok(())
    }

    /// Flush the deferred state: under the direct path the header and
    /// every value have already been written, so this is a no-op. Under
    /// the buffered path, sort entries by tag, write the strategy-
    /// appropriate header, then emit each `(tag-prefix?, value-bytes)`
    /// pair to the parent.
    fn finish(mut self) -> Result<(), RmpError> {
        if !self.buffer {
            return Ok(());
        }
        // Stable sort because tags are unique within a Product (macro
        // guarantees no duplicate tags), so any stability quirk would be
        // a registry bug, not a serialization bug.
        self.entries.sort_by_key(|(tag, _)| *tag);
        write_strategy_header(self.parent.inner.get_mut(), self.entries.len(), self.strategy)?;
        for (tag, bytes) in &self.entries {
            if matches!(self.strategy, EncodingStrategy::Tagged) {
                ser::Serializer::serialize_u8(&mut *self.parent, *tag)?;
            }
            self.parent.inner.get_mut().write_all(bytes).map_err(|e| {
                RmpError::custom(format!("failed to flush buffered field bytes: {e}"))
            })?;
        }
        Ok(())
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
        self.serialize_tag_and_value(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
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
/// order — and the buffer-and-flush in `finish` then writes them on the
/// wire in tag-ascending order.
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
        self.serialize_tag_and_value(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

/// Tuple variant payload (`enum E { ... Pair(u32, bool) }`). Same payload
/// shape and tag-resolution rule as a top-level tuple struct — the variant's
/// `payload` `Product` uses positional names (`"0"`, `"1"`, …). The outer
/// `{variant_tag: ...}` map (1-entry discriminator) was written upfront in
/// `serialize_tuple_variant`; the payload's header is deferred to
/// `finish()` along with every other product's.
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
        self.serialize_tag_and_value(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
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
        self.serialize_tag_and_value(tag, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
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
/// have their own adapter ([`TaggedSerializeProduct`]) because they carry
/// the [`Product`](crate::Product) needed to translate field names into
/// integer tags.
pub struct TaggedSerializeViaParent<'ser, 'a, W: Write> {
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
mod builder_tests {
    //! Tests for `Serializer::new` + `with_default_strategy` +
    //! `with_strategy`. Phase 1 — the strategy state is configured but the
    //! encoder doesn't yet branch on it; these tests verify the *state* is
    //! recorded correctly via `Serializer::strategy_for`. Phase 2 will add
    //! encode-side dispatch and tests for the actual wire shape.

    use super::*;
    use crate::Tagged;

    struct Foo;
    impl MsgpackTagged for Foo {
        const TAGGED: Tagged = Tagged::empty_product();
        fn register_into(reg: &mut TagRegistry) {
            reg.try_insert::<Self>("Foo");
        }
    }

    struct Bar;
    impl MsgpackTagged for Bar {
        const TAGGED: Tagged = Tagged::empty_product();
        fn register_into(reg: &mut TagRegistry) {
            reg.try_insert::<Self>("Bar");
        }
    }

    /// Build a registry that contains every type passed in. Test-local
    /// helper — production registries are built via
    /// `TagRegistry::from_type::<T>()` which seeds the walk from one
    /// top-level type.
    fn registry_of<F: FnOnce(&mut TagRegistry)>(f: F) -> TagRegistry {
        let mut reg = TagRegistry::new();
        f(&mut reg);
        reg
    }

    #[test]
    fn default_strategy_is_tagged() {
        let registry = TagRegistry::from_type::<Foo>();
        let s = Serializer::new(Vec::<u8>::new(), &registry);
        assert_eq!(s.strategy_for::<Foo>(), EncodingStrategy::Tagged);
    }

    #[test]
    fn with_default_strategy_changes_default() {
        let registry = TagRegistry::from_type::<Foo>();
        let s = Serializer::new(Vec::<u8>::new(), &registry)
            .with_default_strategy(EncodingStrategy::Array);
        assert_eq!(s.strategy_for::<Foo>(), EncodingStrategy::Array);
    }

    #[test]
    fn per_type_override_beats_default() {
        let registry = TagRegistry::from_type::<Foo>();
        let s = Serializer::new(Vec::<u8>::new(), &registry)
            .with_default_strategy(EncodingStrategy::Array)
            .with_strategy::<Foo>(EncodingStrategy::Tagged);
        assert_eq!(s.strategy_for::<Foo>(), EncodingStrategy::Tagged);
    }

    #[test]
    fn per_type_overrides_are_independent() {
        let registry = registry_of(|r| {
            Foo::register_into(r);
            Bar::register_into(r);
        });
        let s = Serializer::new(Vec::<u8>::new(), &registry)
            .with_default_strategy(EncodingStrategy::Array)
            .with_strategy::<Foo>(EncodingStrategy::Tagged);
        // Bar has no override; falls back to the (changed) default.
        assert_eq!(s.strategy_for::<Foo>(), EncodingStrategy::Tagged);
        assert_eq!(s.strategy_for::<Bar>(), EncodingStrategy::Array);
    }

    #[test]
    fn last_with_strategy_wins() {
        let registry = TagRegistry::from_type::<Foo>();
        let s = Serializer::new(Vec::<u8>::new(), &registry)
            .with_strategy::<Foo>(EncodingStrategy::Array)
            .with_strategy::<Foo>(EncodingStrategy::Tagged);
        assert_eq!(s.strategy_for::<Foo>(), EncodingStrategy::Tagged);
    }

    /// Setting a strategy for a type the registry never saw is almost
    /// always a type-graph miss bug — the override would silently never
    /// fire. `with_strategy` panics so the misuse surfaces at config
    /// time.
    #[test]
    #[should_panic(expected = "is not in the registry")]
    fn with_strategy_panics_on_unregistered_type() {
        let registry = TagRegistry::from_type::<Foo>();
        let _ = Serializer::new(Vec::<u8>::new(), &registry)
            .with_strategy::<Bar>(EncodingStrategy::Array);
    }

    /// `with_strategy_for_name` is the policy-site sibling — never
    /// asserts. Inserting an override for a name the registry doesn't
    /// know about is a no-op at encode time (the name never matches any
    /// `serialize_struct` call), so `strategy_for::<Foo>` still resolves
    /// the configured default.
    #[test]
    fn with_strategy_for_name_does_not_assert_registration() {
        let registry = TagRegistry::from_type::<Foo>();
        let s = Serializer::new(Vec::<u8>::new(), &registry)
            .with_default_strategy(EncodingStrategy::Array)
            .with_strategy_for_name("Bar", EncodingStrategy::Tagged);
        assert_eq!(s.strategy_for::<Foo>(), EncodingStrategy::Array);
    }
}
