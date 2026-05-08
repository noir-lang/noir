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

This crate does not produce wire bytes yet. The companion
`TaggedMsgpackSerializer` / `TaggedMsgpackDeserializer` (forthcoming) will
read this metadata and translate between serde calls and msgpack bytes.

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

The wrapper will support three encoding strategies, all driven by the
*same* `Tagged` metadata:

| strategy | wire shape | when |
|---|---|---|
| **Tagged** | int-keyed map | top-level types where field churn is expected (`Program`, `Circuit`) |
| **Array** | positional msgpack array | small leaf types where size matters more than evolvability |
| **Named** | string-keyed map | falls back to `rmp_serde` defaults |

Strategy is per-type and picked by the wrapper, not by the macro — every
`#[derive(MsgpackTagged)]` type works under all three. Decode is
shape-agnostic (peek at the next msgpack token, dispatch to the right
reader), so a single binary can read all three formats without a
configuration switch.

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
    pub defaults: &'static [Tag],
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
    pub default_on_reserved: bool,
    pub default_on_unknown: bool,
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
| `#[tag(N, default)]` | wire-tolerant: decoder fills `T::default()` when the tag is missing. Macro emits `where T: Default` |
| `#[tag(skip)]` | drop the field from the wire |
| `#[serde(skip)]` | alias for `#[tag(skip)]` |
| `#[serde(rename = "X")]` | overrides the wire name in `Product.fields` (load-bearing for shadow-DTOs whose wire DTO renames individual fields) |
| `PhantomData<_>` | auto-skipped — no `#[tag]` annotation needed |

Any other untagged field is a compile error. Strict by default — the
macro never silently ignores a field.

### Tuple structs and tuple variants

Two modes, all-or-nothing:

```rust
struct Pair(u32, bool);                                    // implicit positional: (0, "0"), (1, "1")
struct Reordered(#[tag(2)] u32, #[tag(0)] bool, #[tag(1)] u8);  // all-explicit: allows reordering / `default`
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
`Product`; the kind discriminator is what tells the wrapper how to encode.

### Type-level `#[tagged(...)]`

| attribute | applies to | meaning |
|---|---|---|
| `reserved(N, M, ...)` | structs and enums | retire tags so they can't be reused. Compile-time guard, not runtime — see migration guide for runtime behavior |
| `allow_unknown_tags` | structs only | decoder silently skips unknown field tags. Use sparingly: silently swallows real corruption |
| `via(WireType)` | any | shadow-DTO delegation — see below |
| `default_on_reserved` | enums only | substitute `T::default()` for retired variant tags on decode (backwards-compat). Macro emits `where Self: Default` |
| `default_on_unknown` | enums only | substitute `T::default()` for *any* unknown variant tag on decode (forward-compat). Same `Self: Default` bound |

### Variant-level `#[tagged(...)]`

`reserved(...)` and `allow_unknown_tags` apply per-variant payload, with
the same semantics as their type-level counterparts. Sum-level modifiers
(`default_on_*`, `via`) are rejected at the variant level.

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
2. If older readers must accept the new wire, mark the new field
   `#[tag(N, default)]`. Older clients (predating the field) ignore it
   on decode; newer clients always emit it.
3. If older readers should *reject* the new wire, use plain `#[tag(N)]`.
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

### Retiring a variant (enum)

`reserved` on a sum type works the same way for *variant* tags, but the
runtime story is different: a sum can't "skip" an unknown variant tag —
the value's discriminator itself becomes unrepresentable. Two opt-in
recovery flags:

- `#[tagged(default_on_reserved)]` — when decoding hits a retired
  variant tag, produce `T::default()` instead of erroring. Backwards-
  compat for legacy data carrying retired discriminators.
- `#[tagged(default_on_unknown)]` — same fallback for *any* unknown
  variant tag (whether retired or just newer than the local schema
  knows about). Forward-compat. Riskier — silently swallows real
  corruption — so use it only where `T::default()` is a sound
  substitute for "I don't recognize this discriminator" (e.g. metadata-
  bearing types like `InlineType`, **not** execution-critical types
  like `BrilligOpcode`).

Both flags require `Self: Default`. The macro adds the bound
automatically; a missing `derive(Default)` is a compile error.

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
