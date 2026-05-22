# `msgpack_tagged`

Metadata layer for a tagged-map serialization format used by Noir bytecode.
Companion crate to `msgpack_tagged_derive` (the `#[derive(MsgpackTagged)]`
proc-macro).

Design: [issue #12554](https://github.com/noir-lang/noir/issues/12554),
umbrella tracker [#7934](https://github.com/noir-lang/noir/issues/7934).

## What this crate provides

- The `MsgpackTagged` trait — a single associated `const TAGGED: Tagged`
  that captures a type's wire shape, plus a `register_into(&mut TagRegistry)`
  hook that walks the type graph.
- The wire-shape data model: `Tagged` / `Product` / `Sum` / `Variant` / `VariantKind`.
- A `TagRegistry` keyed by serde name, with `TypeId`-based collision
  detection for shadow-DTO patterns.
- Blanket `MsgpackTagged` impls for primitives, the deterministic stdlib
  containers (`Vec`, `BTreeMap`, `BTreeSet`, `Option`, `Box`, arrays,
  tuples up to 12), and `PhantomData<T>`.

Wire bytes are produced by the in-crate `Serializer` (and consumed by the
companion `Deserializer`), thin wrappers around `rmp_serde` that translate
between serde calls and msgpack bytes by consulting the `TagRegistry`. The
public one-shot entry points are:

```rust
let bytes = msgpack_tagged::msgpack_tagged_serialize(&value)?;
let decoded: T = msgpack_tagged::msgpack_tagged_deserialize(&bytes)?;
```

The `Serializer` is feature-complete; the `Deserializer` is currently a
skeleton that forwards to `rmp_serde` and is being filled in shape by shape.

## Wire shape

The intent is **int-keyed msgpack maps with protobuf-style field numbers**:

```text
struct Foo { a: u32, b: bool }   ⇒   {0: <u32>, 1: <bool>}
enum Bar {
    Wibble,                      ⇒   {<variant_tag>: nil}
    Quux(u32),                   ⇒   {<variant_tag>: <u32>}                    // newtype: pass-through
    Quuux(u32, bool),            ⇒   {<variant_tag>: {0: <u32>, 1: <bool>}}    // tuple: positional field map
    Wobble { a: u32, b: bool },  ⇒   {<variant_tag>: {0: <u32>, 1: <bool>}}    // struct: field map
}
```

Tags are `u8` (so they stay in msgpack's `fixint` range at the 1-byte
encoding). Field names never appear on the wire. Adding, removing, or
reordering fields is safe as long as tag values stay stable.

The `Serializer` will support two encoding strategies, both driven by the
*same* `Tagged` metadata:

| strategy | wire shape (struct) | when |
|---|---|---|
| **Tagged** | int-keyed `fixmap` `{0: a, 1: b, …}` | top-level types where field churn is expected (`Program`, `Circuit`) |
| **Array** | positional `fixarray` `[a, b, …]` | leaf types instantiated at scale where the per-field tag byte adds up |

Strategy is per-type and picked by the serializer (builder API on
`Serializer::new` + `with_default_strategy` / `with_strategy`), not by
the macro — every `#[derive(MsgpackTagged)]` type works under both.
Enum variants are always emitted int-keyed regardless of strategy;
strategy only affects struct shape.

The decoder is **not** shape-agnostic. The outer `Format` byte
(`Format::MsgpackTagged`) routes to the tagged reader, and that reader
only handles the two shapes this crate emits — `fixmap` for `Tagged`,
`fixarray` for `Array`. Legacy formats (`Format::Msgpack`,
`Format::MsgpackCompact`) have their own format bytes and go through
`rmp_serde` directly; the tagged decoder never sees them. The C++ side
*does* probe — see the design doc for why the asymmetry is justified.

## The data model

```rust
pub type Tag = u8;

pub enum Tagged {
    Product(Product),  // structs, tuple structs, enum-variant payloads
    Sum(Sum),          // enums
}

pub struct Product {
    pub fields: &'static [(Tag, &'static str)],
    pub reserved: &'static [Tag],
    pub allow_unknown_tags: bool,
}

pub enum VariantKind {
    Unit,     // no payload at all
    Newtype,  // single-element tuple variant — inner value passes through under the variant tag
    Tuple,    // multi-element tuple variant — fields go into a positional map
    Struct,   // named-field variant — fields go into a field map
}

pub struct Variant {
    pub tag: Tag,
    pub name: &'static str,
    pub kind: VariantKind,
    pub payload: Product,  // empty for Unit and Newtype
}

pub struct Sum {
    pub variants: &'static [Variant],
    pub reserved: &'static [Tag],
    pub on_reserved_tag: Option<Tag>,
    pub on_unknown_tag: Option<Tag>,
}

pub trait MsgpackTagged: 'static {
    const TAGGED: Tagged;
    fn register_into(reg: &mut TagRegistry);
}
```

`Tagged` / `Product` / `Sum` / `Variant` are all `Copy` with public
fields — they're built directly in `const` context by the macro.

## Annotating your types

```rust
use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged, serde::Serialize, serde::Deserialize)]
struct Expression {
    #[tag(0)] mul_terms: Vec<MulTerm>,
    #[tag(1)] linear: Vec<LinearTerm>,
    #[tag(2)] q_c: FieldElement,
}

#[derive(MsgpackTagged, serde::Serialize, serde::Deserialize)]
#[tagged(reserved(2, 5))]
enum BinaryFieldOp {
    #[tag(0)] Add,
    #[tag(1)] Sub,
    #[tag(3)] Mul,        // tags 2 and 5 retired; macro forbids reuse
    #[tag(4)] Div,
    #[tag(6)] Equals,
}
```

### Field-level attributes

| attribute | meaning |
|---|---|
| `#[tag(N)]` | required — `N` is the wire tag |
| `#[serde(skip)]` | drop the field from the wire (auto-recognized — the macro reads it directly rather than inventing a duplicate signal) |
| `#[serde(default)]` | wire-tolerant: decoder fills `T::default()` (or a custom function via `#[serde(default = "..."]`) when the tag is missing. Pure serde-derive feature — nothing the macro emits — but the most common companion when retiring tags or adding new ones |
| `#[serde(rename = "X")]` | overrides the wire name in `Product.fields` (load-bearing for shadow-DTOs whose wire DTO renames individual fields) |
| `PhantomData<_>` | auto-skipped — no `#[tag]` annotation needed |

Any other untagged field is a compile error. Strict by default — the
macro never silently ignores a field.

### Tuple structs and tuple variants

Two modes, all-or-nothing:

```rust
struct Pair(u32, bool);                                  // implicit positional: (0, "0"), (1, "1")
struct Reordered(#[tag(2)] u32, #[tag(0)] bool, #[tag(1)] u8);  // all-explicit: allows reordering
```

Mixing implicit and explicit (`#[tag(0)] u32, bool`) is rejected — the
wire layout would be ambiguous.

### Newtype variants

A single-element tuple variant is a *newtype variant* — its wire bytes are
the inner type's bytes, written directly under the variant tag with no
field-level tag map of its own:

```rust
enum E {
    #[tag(0)] Wrap(InnerType),     // ⇒  {0: <InnerType bytes>}
}
```

Newtype variants are pass-through and zero-cost on the wire. Consequently:

- `#[tag(N)]` on the inner field is rejected — there's no field tag space
  to put it in. If you want field-level tagging, use a multi-element tuple
  variant or a struct variant instead.
- Variant-level `#[tagged(reserved(...))]` and `#[tagged(allow_unknown_tags)]`
  are also rejected on newtype (and unit) variants — those flags govern a
  payload field tag space that doesn't exist here.

The metadata distinction lives in `VariantKind` (`Newtype` vs. `Tuple` vs.
`Struct` vs. `Unit`). Both `Unit` and `Newtype` carry an empty `payload`
`Product`; the kind discriminator is what tells the `Serializer` how to encode.

### Type-level `#[tagged(...)]`

| attribute | applies to | meaning |
|---|---|---|
| `reserved(N, M, ...)` | structs and enums | retire tags so they can't be reused. Compile-time guard, not runtime — see migration guide for runtime behavior |
| `allow_unknown_tags` | structs only | decoder silently skips unknown field tags. Use sparingly: silently swallows real corruption |
| `via(WireType)` | any | shadow-DTO delegation — see below |
### Variant-level `#[tagged(...)]`

Two grammar groups apply to variants:

* **Payload-shape modifiers** — `reserved(...)` and `allow_unknown_tags`
  configure the variant's payload, same semantics as their type-level
  counterparts.
* **Fallback-routing markers** — `on_reserved` and `on_unknown` mark a
  unit variant as the routing target on the enclosing enum for retired
  / unknown wire tags respectively. See the "Retiring a variant" section
  for the runtime semantics.

`via(...)` is type-level only and rejected on variants.

```rust
#[derive(MsgpackTagged)]
enum Lenient {
    #[tag(0)]
    #[tagged(reserved(5), allow_unknown_tags)]
    Multi {
        #[tag(0)] a: u32,
        #[tag(1)] b: bool,
        // tag 5 retired; unknown tags inside this variant's payload are silently skipped
    },
}
```

### Shadow-DTO via `#[tagged(via(WireType))]`

When the public type has a hand-written `Serialize`/`Deserialize` that
bridges through a wire-shaped DTO, only the wire DTO carries `#[tag(N)]`
annotations. The public type delegates:

```rust
#[derive(MsgpackTagged)]                       // satisfies the bound chain
#[tagged(via(CircuitWire<F>))]                 // delegate the wire shape
pub struct Circuit<F> {
    pub current_witness_index: u32,            // internal-only, not on the wire
    pub opcodes: Vec<Opcode<F>>,
    // no #[tag(N)] anywhere — fields are wire-irrelevant
}

#[derive(MsgpackTagged, serde::Serialize, serde::Deserialize)]
#[serde(rename = "Circuit")]                   // register under the public name
struct CircuitWire<F> {
    #[tag(0)] opcodes: Vec<Opcode<F>>,
    #[tag(1)] private_parameters: BTreeSet<Witness>,
    // ... wire-shape fields
}
```

`#[tagged(via(...))]` emits an empty `Tagged::Product`, suppresses the
"untagged field" compile error on the public type, and delegates
`register_into` to the wire DTO.

## Migration guide

### Adding a new field

1. Pick the next unused tag (and not in any `reserved(...)` list).
2. If newer readers must tolerate the *older* wire (which doesn't carry
   this tag), pair `#[tag(N)]` with `#[serde(default)]` — serde-derive's
   standard default-filling kicks in when the tag is missing on decode.
   Older clients (predating the field) silently ignore the value if
   they opt in via `#[tagged(allow_unknown_tags)]`.
3. If newer readers should *reject* the older wire, use plain `#[tag(N)]`.
   The macro will accept the type either way — the choice is purely a
   compatibility decision.

### Retiring an existing field (struct)

1. Remove the field from the type.
2. Add the field's tag to `#[tagged(reserved(N))]` (or extend the existing
   list).
3. Compile fails if any other field tries to reuse the retired tag —
   that's the protection.

Decoders processing legacy wire bytes will hit the retired tag. Without
`allow_unknown_tags` they error; with it they silently skip the value.
Pick based on whether dropping the field could change application
semantics.

**Interaction with `Array` strategy.** Under `Array` the wire is
positional and only carries active fields, but the decoder walks a
merged-sorted `(active + reserved)` layout so legacy `Array` data (V1,
written before the retirement) still decodes — the reserved slot drains
the now-orphaned V1 value silently. That alignment is fragile in one
direction: if a reserved tag has any active tag *after* it in tag
order, a V2-on-V2 round-trip under `Array` would drain the wire byte the
encoder wrote for the next active field. To keep this safe the encoder
**auto-downgrades** the product to `Tagged` whenever it detects a
non-strictly-trailing reserved tag — the int-keyed-map shape is
self-describing per entry, so alignment isn't at risk. The downgrade is
silent and local to that product; other types in the same serializer
keep their configured strategy.

```text
fields: [(0, "a"), (2, "c")]   reserved: [1]    →  reserved is interleaved → Array auto-downgrades to Tagged
fields: [(0, "a"), (1, "b")]   reserved: [9]    →  reserved is strictly trailing → Array preserved
```

In the trailing case the wire stays compact (a `fixarray` of the active
values) and a later V3 that adds a new field at a tag higher than the
reserved one — paired with `#[serde(default)]` — picks up the missing
value via serde-derive's standard short-wire fill. If you need a
middle-of-shape retirement and still want compact wire bytes, the
practical move is to keep the type on `Tagged` going forward.

### Retiring a variant (enum)

`reserved` on a sum type works the same way for *variant* tags, but the
runtime story is different: a sum can't "skip" an unknown variant tag —
the value's discriminator itself becomes unrepresentable. The recovery
hook is a unit variant marked as a fallback routing target. The marker
itself is the opt-in — no separate type-level flag:

- `#[tagged(on_reserved)]` — when decoding hits a retired variant tag
  (one listed in `reserved(...)`), route to this variant. Backward-compat
  for legacy data carrying retired discriminators.
- `#[tagged(on_unknown)]` — same routing for any variant tag that's in
  neither `variants` nor `reserved`. Forward-compat. Riskier — silently
  swallows real corruption — so use it only where this fallback is a
  sound substitute for "I don't recognize this discriminator" (e.g.
  metadata-bearing types like `InlineType`, **not** execution-critical
  types like `BrilligOpcode`).

The two markers are independent axes: marking only `on_reserved`
accepts retired tags but errors on truly unknown ones; marking only
`on_unknown` does the reverse. For unified "any unknown tag goes here"
behavior, put both attrs on a single variant:

```rust
#[derive(MsgpackTagged)]
#[tagged(reserved(2))]
enum Op {
    #[tag(0)] Real,
    #[tag(9)]
    #[tagged(on_reserved, on_unknown)]
    Unrecognized,
}
```

Both markers require a unit variant (the wire payload is discarded on
routing, so the variant can't carry one of its own). At most one variant
per marker kind. The fallback variant has its own wire tag and
round-trips like any other unit variant.

### Renaming a field

If the wire shape can stay the same: change only the Rust ident. The
wire name in `Product.fields` follows the ident, so the wire bytes are
unaffected by Rust-level renames.

If the *wire name* must change (e.g. a shadow-DTO wire field is renamed
in serde): use `#[serde(rename = "X")]` on the wire DTO field. The
macro picks up the rename target.

### Renaming a type

Same rules: type-level `#[serde(rename = "X")]` controls the registry
key. The shadow-DTO pattern leans on this — the wire DTO is renamed to
match the public type's name so consumers serializing the public type
hit the wire DTO's metadata.

## Determinism

Field ordering on the wire is **tag-ascending**, not source-declaration
order — so two semantically-equal structs encode to byte-identical
output regardless of how the user laid out the fields. `BTreeMap` /
`BTreeSet` are the only blanket-impl'd associative containers; `HashMap`
/ `HashSet` are deliberately omitted because their iteration order is
non-deterministic.

### Reordering has a hidden encode-time cost

The encoder takes a fast path when a type's source-declaration order is
already tag-ascending (the common case — newly-added types, types using
implicit positional tags, types with `#[tag(N)]` values in source
order): each field streams straight through to the output, no per-field
allocation.

When source order *doesn't* match tag order — typically because the
type has been schema-evolved (a tag retired and re-added at a higher
number, fields reshuffled for readability, etc.) — the encoder falls
back to a buffer-and-sort path: each field's bytes go into a
per-field `Vec<u8>`, then the entries are sorted by tag and flushed in
canonical order at the end of the struct. This preserves the
tag-ascending wire-order promise above, but the per-field allocation
is an O(field-count) cost paid on every value of that type.

Practically: **if you reorder fields to make the source readable, you
opt into a per-field allocation at encode time**. For a hot leaf type
instantiated thousands of times in a single program, that adds up. The
fix is either to keep `#[tag(N)]` annotations in source order (the
encoder then takes the fast path), or to accept the cost in exchange
for the source-layout flexibility. The `Array` strategy *requires* the
buffer for correctness on reordered types — it's not optional there —
but `Tagged` types pay it only for the byte-determinism guarantee, so
if you've got a Tagged type where cross-implementation byte-equivalence
isn't load-bearing, keeping the source ordered avoids the cost.

## Consuming from C++

The ACIR crate generates C++ `msgpack_unpack` methods for every wire
type, used by Barretenberg to deserialize ACIR / Witness payloads. The
generator (`acvm-repo/acir/src/lib.rs`, `serde_acir_cpp_codegen` /
`serde_witness_map_cpp_codegen`) emits a four-way dispatch per
struct:

| `o.type` / first key | format(s) that produce it |
|---|---|
| `MAP`, first key is `POSITIVE_INTEGER` | `MsgpackTagged` (Tagged variant header; Tagged-strategy struct) |
| `MAP`, first key is `STR` | legacy `Msgpack` (string-keyed structs and enum variants) |
| `STR` (top-level) | legacy `MsgpackCompact` unit variant (bare variant name) |
| `ARRAY` | legacy `MsgpackCompact` struct **and** `MsgpackTagged::Array` struct |

The last row is the one to watch.

### The `ARRAY` ambiguity

Both `Format::MsgpackCompact` and `Format::MsgpackTagged` with the
`Array` strategy produce a bare `fixarray` of field values on the
wire — identical shapes, no per-element metadata. They differ only in
**order**:

* `MsgpackCompact` emits fields in *source-declaration* order
  (`rmp_serde::with_struct_tuple()` behavior).
* `MsgpackTagged::Array` emits fields in *tag-ascending* order
  (`Serializer::with_default_strategy(Array)`).

The Rust side knows which format is being decoded because the outer
`Format` byte is consumed before dispatch lands in `msgpack_unpack`. The
C++ side **doesn't** — `msgpack_unpack(msgpack::object const& o)`
receives the parsed inner object with no surviving reference to the
format byte. Both shapes look like `msgpack::type::ARRAY` to it.

### The fix: source order must equal tag order

The generated C++ assumes `ARRAY` means *tag-ascending* — that's the
only positional decode the codegen emits. For this to also be correct
under `MsgpackCompact`, the producing Rust type's
**source-declaration order must be tag-ascending** as well. The two
orders then agree, both wires are byte-identical, and the C++ decoder
parses them with the same logic.

The codegen enforces this at test time: `serde_acir_cpp_codegen` panics
if any type reachable from `Program` / `Circuit` / `WitnessMap` /
`WitnessStack` has `tag_order_matches_source == false`. Reorder the
Rust fields so `#[tag(N)]` values increase in source order, or drop the
type from the C++ wire types.

### Workaround: reorder in the model via shadow DTO

If the Rust-side type really needs a different field order than the
wire — typically for ergonomics, accessor placement, or aligning with
unrelated traits — wrap the wire identity in a shadow DTO. The public
type's source layout is yours to choose; the DTO carries the canonical
tag-ascending order, and `#[tagged(via(WireType))]` redirects the
serialization path through it:

```rust
#[derive(Serialize, Deserialize, MsgpackTagged)]
#[tagged(via(FooWire))]
pub struct Foo {
    // Order whatever way reads best for callers.
    pub b: bool,
    pub a: u32,
}

#[derive(Serialize, Deserialize, MsgpackTagged)]
#[serde(rename = "Foo")]
struct FooWire {
    // Stays tag-ascending: matches `MsgpackCompact`'s source-order
    // output and the C++ codegen's positional-array assumption.
    #[tag(0)] a: u32,
    #[tag(1)] b: bool,
}
```

The codegen-time order check sees `FooWire` (registered under the same
serde name `"Foo"` thanks to `#[serde(rename)]`), so the invariant
holds and C++ generation succeeds — even though `Foo`'s source layout
isn't tag-ascending. See the
[Shadow-DTO via `#[tagged(via(WireType))]`](#shadow-dto-via-taggedviawiretype)
section above for the pattern's general form and trade-offs.
