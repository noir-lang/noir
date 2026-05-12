//! Serialization formats we consider using for the bytecode and the witness stack.

use acir_field::FieldElement;
use msgpack_tagged::{
    EncodingStrategy, MsgpackTagged, Serializer as TaggedSerializer, TagRegistry,
    msgpack_tagged_deserialize,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::EnumString;

use crate::circuit::{Circuit, Program, brillig::BrilligBytecode};

const FORMAT_ENV_VAR: &str = "NOIR_SERIALIZATION_FORMAT";

/// A marker byte for the serialization format.
#[derive(Debug, Default, Clone, Copy, IntoPrimitive, TryFromPrimitive, EnumString, PartialEq, Eq)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum Format {
    /// Msgpack with named structs.
    Msgpack = 2,
    /// Msgpack with tuple structs.
    #[default]
    MsgpackCompact = 3,
    /// Msgpack with int-keyed tag maps via the `msgpack_tagged` wrapper —
    /// compact (single-byte field/variant tags) and schema-evolution-friendly
    /// (retiring a field/variant + `#[tagged(reserved(...))]` keeps old data
    /// decodable). Requires `T: MsgpackTagged`.
    MsgpackTagged = 4,
}

impl Format {
    /// Look for a `NOIR_SERIALIZATION_FORMAT` env var to turn on formatted serialization.
    ///
    /// The reason we use an env var is because:
    /// 1. It has to be picked up in methods like `Program::serialize_program_base64` where no config is available.
    /// 2. At the moment this is mostly for testing, to be able to commit code that _can_ produce different formats,
    ///    but only activate it once a version of `bb` that can handle it is released.
    pub fn from_env() -> Result<Option<Self>, String> {
        let Ok(format) = std::env::var(FORMAT_ENV_VAR) else {
            return Ok(None);
        };
        Self::from_str(&format)
            .map(Some)
            .map_err(|e| format!("unknown format '{format}' in {FORMAT_ENV_VAR}: {e}"))
    }
}

/// Serialize a value using MessagePack, based on `serde`.
///
/// This format is compact can be configured to be backwards compatible.
///
/// When `compact` is `true`, it serializes structs as tuples, otherwise it writes their field names.
/// Enums are always serialized with their variant names (despite what the library comments say, and it's not configurable).
///
/// Set `compact` to `true` if we want old readers to fail when a new field is added to a struct,
/// that is, if we think that ignoring a new field could lead to incorrect behavior.
pub(crate) fn msgpack_serialize<T: Serialize>(
    value: &T,
    compact: bool,
) -> std::io::Result<Vec<u8>> {
    // There are convenience methods to serialize structs as tuples or maps:
    // * `rmp_serde::to_vec` uses tuples
    // * `rmp_serde::to_vec_named` uses maps
    // However it looks like the default `BytesMode` is not compatible with the C++ deserializer,
    // so we have to use `rmp_serde::Serializer` directly.
    let mut buf = Vec::new();

    let serializer = rmp_serde::Serializer::new(&mut buf)
        .with_bytes(rmp_serde::config::BytesMode::ForceIterables);

    let result = if compact {
        value.serialize(&mut serializer.with_struct_tuple())
    } else {
        value.serialize(&mut serializer.with_struct_map())
    };

    match result {
        Ok(()) => Ok(buf),
        Err(e) => Err(std::io::Error::other(e)),
    }
}

/// Deserialize a value using MessagePack, based on `serde`.
pub(crate) fn msgpack_deserialize<T: for<'a> Deserialize<'a>>(buf: &[u8]) -> std::io::Result<T> {
    rmp_serde::from_slice(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

/// Deserialize any of the supported formats.
pub(crate) fn deserialize_any_format<T>(buf: &[u8]) -> std::io::Result<T>
where
    T: for<'a> Deserialize<'a> + MsgpackTagged,
{
    let Some(format_byte) = buf.first() else {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty buffer"));
    };

    match Format::try_from(*format_byte) {
        Ok(Format::Msgpack) | Ok(Format::MsgpackCompact) => msgpack_deserialize(&buf[1..]),
        Ok(Format::MsgpackTagged) => msgpack_tagged_deserialize(&buf[1..]),
        Err(msg) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, msg.to_string())),
    }
}

pub(crate) fn serialize_with_format<T>(value: &T, format: Format) -> std::io::Result<Vec<u8>>
where
    T: Serialize + MsgpackTagged,
{
    // It would be more efficient to skip having to create a vector here, and use a std::io::Writer instead.
    let mut buf = match format {
        Format::Msgpack => msgpack_serialize(value, false)?,
        Format::MsgpackCompact => msgpack_serialize(value, true)?,
        Format::MsgpackTagged => msgpack_tagged_serialize_acir(value)?,
    };
    let mut res = vec![format.into()];
    res.append(&mut buf);
    Ok(res)
}

/// Serialize a value through the `msgpack_tagged` wrapper with the ACIR
/// policy applied: by default everything reachable from `value` rides on
/// `EncodingStrategy::Array` (the compact positional shape), and the
/// top-level container types (`Program`, `Circuit`, `BrilligBytecode`)
/// flip to `EncodingStrategy::Tagged` so they stay schema-evolvable. The
/// overrides are passed with `must_exist = false`, so the same call
/// works when `value` doesn't reach any of those containers (e.g. a
/// `WitnessMap` or a bare leaf type) — unreachable names get a stray
/// override entry that's never looked up at encode time.
///
/// The strategy override is keyed by the type's **serde name**, not its
/// `TypeId`, so it doesn't matter which field flavor (`FieldElement`,
/// `GenericFieldElement<X>`, …) the caller is using — every
/// `Program<_>` shares the name "Program" and resolves to the same
/// override. The same name-based lookup is what makes the
/// `Circuit`/`CircuitWire` shadow-DTO pattern transparent here:
/// CircuitWire registers under "Circuit" via `#[serde(rename = "Circuit")]`
/// and the override fires on it.
pub(crate) fn msgpack_tagged_serialize_acir<T>(value: &T) -> std::io::Result<Vec<u8>>
where
    T: ?Sized + Serialize + MsgpackTagged,
{
    /// Field flavor used at the policy site to look up strategy overrides
    /// for the generic ACIR containers (`Program<F>`, `Circuit<F>`,
    /// `BrilligBytecode<F>`). The actual `F` here doesn't matter — the
    /// override is matched by serde name via [`type_name_basename`], which
    /// strips the generic parameter. Pinning it to `FieldElement` just
    /// satisfies the `T: MsgpackTagged` bound on `with_strategy::<T>`.
    ///
    /// [`type_name_basename`]: msgpack_tagged::type_name_basename
    type AcirF = FieldElement;

    let registry = TagRegistry::from_type::<T>();
    let mut buf = Vec::new();
    let mut serializer = TaggedSerializer::new(&mut buf, &registry)
        .with_default_strategy(EncodingStrategy::Array)
        .with_strategy::<Program<AcirF>>(EncodingStrategy::Tagged, false)
        .with_strategy::<Circuit<AcirF>>(EncodingStrategy::Tagged, false)
        .with_strategy::<BrilligBytecode<AcirF>>(EncodingStrategy::Tagged, false);
    value.serialize(&mut serializer).map_err(std::io::Error::other)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use brillig::{BitSize, HeapArray, IntegerBitSize, ValueOrArray, lengths::SemiFlattenedLength};
    use std::str::FromStr;

    use crate::{
        native_types::Witness,
        serialization::{Format, msgpack_deserialize, msgpack_serialize},
    };

    mod version1 {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
        pub(crate) enum Foo {
            Case0 { d: u32 },
            Case1 { a: u64, b: bool },
            Case2 { a: i32 },
            Case3 { a: bool },
            Case4 { a: Box<Foo> },
            Case5 { a: u32, b: Option<u32> },
        }
    }

    mod version2 {
        use serde::{Deserialize, Serialize};

        // Removed variants and fields
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
        pub(crate) enum Foo {
            // removed
            // Case0 { .. },
            // unchanged, but position shifted
            Case1 {
                a: u64,
                b: bool,
            },
            // new prefix field
            Case2 {
                b: String,
                a: i32,
            },
            // new suffix field
            Case3 {
                a: bool,
                b: String,
            },
            // reordered, optional removed
            Case5 {
                a: u32,
            },
            // reordered, field renamed
            Case4 {
                #[serde(rename = "a")]
                c: Box<Foo>,
            },
            // new
            Case6 {
                b: i64,
            },
            // new, now more variants than before
            Case7 {
                c: bool,
            },
        }
    }

    /// Test that the `msgpack_serialize(compact=false)` is backwards compatible:
    /// * removal of an enum variant (e.g. opcode no longer in use)
    /// * struct fields added: the old reader ignores new fields, but this could potentially lead to invalid behavior
    /// * struct fields reordered: trivial because fields are named
    /// * struct fields renamed: this would work with positional encoding, or using `#[serde(rename)]`
    #[test]
    fn msgpack_serialize_backwards_compatibility() {
        let cases = vec![
            (version2::Foo::Case1 { b: true, a: 1 }, version1::Foo::Case1 { b: true, a: 1 }),
            (version2::Foo::Case2 { b: "prefix".into(), a: 2 }, version1::Foo::Case2 { a: 2 }),
            (
                version2::Foo::Case3 { a: true, b: "suffix".into() },
                version1::Foo::Case3 { a: true },
            ),
            (
                version2::Foo::Case4 { c: Box::new(version2::Foo::Case1 { a: 4, b: false }) },
                version1::Foo::Case4 { a: Box::new(version1::Foo::Case1 { a: 4, b: false }) },
            ),
            (version2::Foo::Case5 { a: 5 }, version1::Foo::Case5 { a: 5, b: None }),
        ];

        for (i, (v2, v1)) in cases.into_iter().enumerate() {
            let bz = msgpack_serialize(&v2, false).unwrap();
            let v = msgpack_deserialize::<version1::Foo>(&bz)
                .unwrap_or_else(|e| panic!("case {i} failed: {e}"));
            assert_eq!(v, v1);
        }
    }

    /// Test that the `msgpack_serialize(compact=true)` is backwards compatible for a subset of the cases:
    /// * removal of an enum variant (e.g. opcode no longer in use)
    /// * struct fields renamed: accepted because position based
    /// * adding unused enum variants
    ///
    /// And rejects cases which could lead to unintended behavior:
    /// * struct fields added: rejected because the number of fields change
    /// * struct fields reordered: rejected because fields are position based
    #[test]
    fn msgpack_serialize_compact_backwards_compatibility() {
        let cases = vec![
            (version2::Foo::Case1 { b: true, a: 1 }, version1::Foo::Case1 { b: true, a: 1 }, None),
            (
                version2::Foo::Case2 { b: "prefix".into(), a: 2 },
                version1::Foo::Case2 { a: 2 },
                Some("wrong msgpack marker FixStr(6)"),
            ),
            (
                version2::Foo::Case3 { a: true, b: "suffix".into() },
                version1::Foo::Case3 { a: true },
                Some("array had incorrect length, expected 1"),
            ),
            (
                version2::Foo::Case4 { c: Box::new(version2::Foo::Case1 { a: 4, b: false }) },
                version1::Foo::Case4 { a: Box::new(version1::Foo::Case1 { a: 4, b: false }) },
                None,
            ),
            (
                version2::Foo::Case5 { a: 5 },
                version1::Foo::Case5 { a: 5, b: None },
                Some("invalid length 1, expected struct variant Foo::Case5 with 2 elements"),
            ),
        ];

        for (i, (v2, v1, ex)) in cases.into_iter().enumerate() {
            let bz = msgpack_serialize(&v2, true).unwrap();
            let res = msgpack_deserialize::<version1::Foo>(&bz);
            match (res, ex) {
                (Ok(v), None) => {
                    assert_eq!(v, v1);
                }
                (Ok(_), Some(ex)) => panic!("case {i} expected to fail with {ex}"),
                (Err(e), None) => panic!("case {i} expected to pass; got {e}"),
                (Err(e), Some(ex)) => {
                    let e = e.to_string();
                    if !e.contains(ex) {
                        panic!("case {i} error expected to contain {ex}; got {e}")
                    }
                }
            }
        }
    }

    /// Test that an enum where each member wraps a struct serializes as a single item map keyed by the type.
    #[test]
    fn msgpack_repr_enum_of_structs() {
        use rmpv::Value; // cSpell:disable-line

        let value = ValueOrArray::HeapArray(HeapArray {
            pointer: brillig::MemoryAddress::Relative(0),
            size: SemiFlattenedLength(3),
        });
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap(); // cSpell:disable-line

        let Value::Map(fields) = msg else {
            panic!("expected Map: {msg:?}");
        };
        assert_eq!(fields.len(), 1);
        let Value::String(key) = &fields[0].0 else {
            panic!("expected String key: {fields:?}");
        };
        assert_eq!(key.as_str(), Some("HeapArray"));
    }

    /// Test that an enum of unit structs serializes as a string.
    #[test]
    fn msgpack_repr_enum_of_unit_structs() {
        let value = IntegerBitSize::U1;
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap(); // cSpell:disable-line

        assert_eq!(msg.as_str(), Some("U1"));
    }

    /// Test how an enum where some members are unit structs serializes.
    #[test]
    fn msgpack_repr_enum_of_mixed() {
        let value = vec![BitSize::Field, BitSize::Integer(IntegerBitSize::U64)];
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap(); // cSpell:disable-line

        assert_eq!(format!("{msg}"), r#"["Field", {"Integer": "U64"}]"#);
    }

    /// Test that a newtype, just wrapping a value, is serialized as the underlying value.
    #[test]
    fn msgpack_repr_newtype() {
        use rmpv::Value; // cSpell:disable-line

        let value = Witness(1);
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap(); // cSpell:disable-line

        assert!(matches!(msg, Value::Integer(_)));
    }

    #[test]
    fn format_from_str() {
        assert_eq!(Format::from_str("msgpack-compact").unwrap(), Format::MsgpackCompact);
        assert_eq!(Format::from_str("msgpack-tagged").unwrap(), Format::MsgpackTagged);
    }

    /// `Format::MsgpackTagged` round-trips a value through `msgpack_tagged`'s
    /// int-keyed-map encoder/decoder, with the `Format` byte prepended so
    /// `deserialize_any_format` can route to the right decoder. The fixture
    /// type derives `MsgpackTagged` so the bound on the dispatch functions
    /// is satisfied.
    #[test]
    fn msgpack_tagged_format_roundtrip() {
        use msgpack_tagged::MsgpackTagged;

        #[derive(serde::Serialize, serde::Deserialize, MsgpackTagged, PartialEq, Eq, Debug)]
        struct Foo {
            #[tag(0)]
            a: u32,
            #[tag(1)]
            b: bool,
        }

        let value = Foo { a: 7, b: true };
        let bytes = super::serialize_with_format(&value, Format::MsgpackTagged).unwrap();
        assert_eq!(bytes[0], Format::MsgpackTagged as u8);
        let decoded: Foo = super::deserialize_any_format(&bytes).unwrap();
        assert_eq!(decoded, value);
    }

    /// `msgpack_tagged_serialize_acir` applies the ACIR policy:
    /// `Program` / `Circuit` use the `Tagged` (`fixmap`) shape — they're
    /// the top-level schema-evolution surface — while every other reachable
    /// type defaults to `Array` (`fixarray`). This test exercises both
    /// halves: the outer `Program` byte should be a fixmap marker, and a
    /// nested leaf like `Expression` (reached via `Opcode::AssertZero`)
    /// should be a fixarray.
    #[test]
    fn msgpack_tagged_acir_policy_program_tagged_expression_array() {
        use crate::circuit::{Circuit, Opcode, Program};
        use crate::native_types::Expression;
        use acir_field::FieldElement;
        use rmpv::Value;

        let expr: Expression<FieldElement> = Expression {
            mul_terms: vec![],
            linear_combinations: vec![],
            q_c: FieldElement::from(1u128),
        };
        let circuit: Circuit<FieldElement> = Circuit {
            function_name: "main".to_string(),
            opcodes: vec![Opcode::AssertZero(expr)],
            ..Circuit::default()
        };
        let program =
            Program::<FieldElement> { functions: vec![circuit], unconstrained_functions: vec![] };

        let bytes = super::msgpack_tagged_serialize_acir(&program).expect("encode succeeds");
        let value = rmpv::decode::read_value(&mut bytes.as_slice()).expect("valid msgpack");

        // Outer Program is fixmap (Tagged): the function asked for `Array`
        // by default, but the override flipped Program/Circuit back to
        // Tagged.
        let Value::Map(program_entries) = &value else {
            panic!("expected fixmap for Program under the ACIR policy, got {value:?}");
        };
        // Both Program tags (0: functions, 1: unconstrained_functions) on
        // the wire, int-keyed.
        assert_eq!(program_entries.len(), 2);
        assert!(program_entries.iter().all(|(k, _)| matches!(k, Value::Integer(_))));

        // Drill into the first function (a Circuit) — also fixmap.
        let functions = program_entries
            .iter()
            .find(|(k, _)| k.as_u64() == Some(0))
            .map(|(_, v)| v)
            .expect("functions tag present");
        let Value::Array(functions) = functions else {
            panic!("functions field should be a msgpack array, got {functions:?}");
        };
        let Value::Map(circuit_entries) = &functions[0] else {
            panic!("expected fixmap for Circuit, got {:?}", functions[0]);
        };
        assert!(circuit_entries.iter().all(|(k, _)| matches!(k, Value::Integer(_))));

        // Drill into the opcodes and pull the AssertZero variant payload —
        // that payload is the bare `Expression`. Expression has no
        // override, so it should land on Array → fixarray.
        let opcodes_value = circuit_entries
            .iter()
            .find(|(k, _)| k.as_u64() == Some(2))
            .map(|(_, v)| v)
            .expect("opcodes tag present on Circuit wire");
        let Value::Array(opcodes) = opcodes_value else {
            panic!("opcodes should be a msgpack array, got {opcodes_value:?}");
        };
        // Each opcode is a `{variant_tag: payload}` map; AssertZero's
        // payload is the Expression value, which under the ACIR policy
        // should be a fixarray.
        let Value::Map(opcode) = &opcodes[0] else {
            panic!("opcode should be a 1-entry map, got {:?}", opcodes[0]);
        };
        assert_eq!(opcode.len(), 1);
        let expression_value = &opcode[0].1;
        assert!(
            matches!(expression_value, Value::Array(_)),
            "expected fixarray for nested Expression under default Array policy, got \
             {expression_value:?}",
        );
    }

    /// Sanity: the policy round-trips correctly through `Format::MsgpackTagged`.
    /// Catches alignment bugs between the encode-side policy and the
    /// shape-peeking decoder.
    #[test]
    fn msgpack_tagged_acir_policy_program_roundtrips_through_format() {
        use crate::circuit::Program;
        use acir_field::FieldElement;

        let program = Program::<FieldElement>::default();
        let bytes = super::serialize_with_format(&program, Format::MsgpackTagged).unwrap();
        let decoded: Program<FieldElement> = super::deserialize_any_format(&bytes).unwrap();
        assert_eq!(decoded, program);
    }
}
