//! Serialization formats we consider using for the bytecode and the witness stack.

use crate::proto::convert::ProtoSchema;
use acir_field::AcirField;
use noir_protobuf::ProtoCodec;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::EnumString;

const FORMAT_ENV_VAR: &str = "NOIR_SERIALIZATION_FORMAT";

/// A marker byte for the serialization format.
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub(crate) enum Format {
    /// Bincode without format marker.
    /// This does not actually appear in the data.
    BincodeLegacy = 0,
    /// Bincode with format marker.
    Bincode = 1,
    /// Msgpack with named structs.
    Msgpack = 2,
    /// Msgpack with tuple structs.
    MsgpackCompact = 3,
    Protobuf = 4,
}

impl Format {
    /// Look for a `NOIR_SERIALIZATION_FORMAT` env var to turn on formatted serialization.
    ///
    /// The reason we use an env var is because:
    /// 1. It has to be picked up in methods like `Program::serialize_program_base64` where no config is available.
    /// 2. At the moment this is mostly for testing, to be able to commit code that _can_ produce different formats,
    ///     but only activate it once a version of `bb` that can handle it is released.
    pub(crate) fn from_env() -> Result<Option<Self>, String> {
        let Ok(format) = std::env::var(FORMAT_ENV_VAR) else {
            return Ok(None);
        };
        Self::from_str(&format)
            .map(Some)
            .map_err(|e| format!("unknown format '{format}' in {FORMAT_ENV_VAR}: {e}"))
    }
}

/// Serialize a value using `bincode`, based on `serde`.
///
/// This format is compact, but provides no backwards compatibility.
pub(crate) fn bincode_serialize<T: Serialize>(value: &T) -> std::io::Result<Vec<u8>> {
    bincode::serialize(value).map_err(std::io::Error::other)
}

/// Deserialize a value using `bincode`, based on `serde`.
pub(crate) fn bincode_deserialize<T: for<'a> Deserialize<'a>>(buf: &[u8]) -> std::io::Result<T> {
    bincode::deserialize(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
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
#[allow(dead_code)]
pub(crate) fn msgpack_serialize<T: Serialize>(
    value: &T,
    compact: bool,
) -> std::io::Result<Vec<u8>> {
    if compact {
        // The default behavior encodes struct fields as
        rmp_serde::to_vec(value).map_err(std::io::Error::other)
    } else {
        // Or this to be able to configure the serialization:
        // * `Serializer::with_struct_map` encodes structs with field names instead of positions, which is backwards compatible when new fields are added, or optional fields removed.
        // * consider using `Serializer::with_bytes` to force buffers to be compact, or use `serde_bytes` on the field.
        // * enums have their name encoded in `Serializer::serialize_newtype_variant`, but originally it was done by index instead
        rmp_serde::to_vec_named(value).map_err(std::io::Error::other)
    }
}

/// Deserialize a value using MessagePack, based on `serde`.
#[allow(dead_code)]
pub(crate) fn msgpack_deserialize<T: for<'a> Deserialize<'a>>(buf: &[u8]) -> std::io::Result<T> {
    rmp_serde::from_slice(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

/// Serialize a value using `protobuf`.
///
/// This format is forwards and backwards compatible, but requires code generation based on `.proto` schemas.
#[allow(dead_code)]
pub(crate) fn proto_serialize<F, T, R>(value: &T) -> Vec<u8>
where
    F: AcirField,
    R: prost::Message,
    ProtoSchema<F>: ProtoCodec<T, R>,
{
    ProtoSchema::<F>::serialize_to_vec(value)
}

/// Deserialize a value using `protobuf`.
#[allow(dead_code)]
pub(crate) fn proto_deserialize<F, T, R>(buf: &[u8]) -> std::io::Result<T>
where
    F: AcirField,
    R: prost::Message + Default,
    ProtoSchema<F>: ProtoCodec<T, R>,
{
    ProtoSchema::<F>::deserialize_from_slice(buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

/// Deserialize any of the supported formats. Try go guess the format based on the first byte,
/// but fall back to the legacy `bincode` format if anything fails.
pub(crate) fn deserialize_any_format<F, T, R>(buf: &[u8]) -> std::io::Result<T>
where
    T: for<'a> Deserialize<'a>,
    F: AcirField,
    R: prost::Message + Default,
    ProtoSchema<F>: ProtoCodec<T, R>,
{
    // Unfortunately as long as we have to deal with legacy bincode format we might be able
    // to deserialize any other format as pure coincidence, when it was just legacy data.
    // Since `bincode` is the least backwards compatible, let's try that first.
    let bincode_result = bincode_deserialize(buf);

    if bincode_result.is_err() && !buf.is_empty() {
        if let Ok(format) = Format::try_from(buf[0]) {
            match format {
                Format::BincodeLegacy => {
                    // This is just a coincidence, as this format does not appear in the data,
                    // but we know it's none of the other formats.
                }
                Format::Bincode => {
                    if let Ok(value) = bincode_deserialize(&buf[1..]) {
                        return Ok(value);
                    }
                }
                Format::Msgpack | Format::MsgpackCompact => {
                    if let Ok(value) = msgpack_deserialize(&buf[1..]) {
                        return Ok(value);
                    }
                }
                Format::Protobuf => {
                    if let Ok(value) = proto_deserialize(&buf[1..]) {
                        return Ok(value);
                    }
                }
            }
        }
    }

    bincode_result
}

pub(crate) fn serialize_with_format<F, T, R>(value: &T, format: Format) -> std::io::Result<Vec<u8>>
where
    F: AcirField,
    T: Serialize,
    R: prost::Message,
    ProtoSchema<F>: ProtoCodec<T, R>,
{
    // It would be more efficient to skip having to create a vector here, and use a std::io::Writer instead.
    let mut buf = match format {
        Format::BincodeLegacy => return bincode_serialize(value),
        Format::Bincode => bincode_serialize(value)?,
        Format::Protobuf => proto_serialize(value),
        Format::Msgpack => msgpack_serialize(value, false)?,
        Format::MsgpackCompact => msgpack_serialize(value, true)?,
    };
    let mut res = vec![format.into()];
    res.append(&mut buf);
    Ok(res)
}

pub(crate) fn serialize_with_format_from_env<F, T, R>(value: &T) -> std::io::Result<Vec<u8>>
where
    F: AcirField,
    T: Serialize,
    R: prost::Message,
    ProtoSchema<F>: ProtoCodec<T, R>,
{
    match Format::from_env() {
        Ok(Some(format)) => {
            // This will need a new `bb` even if it's the bincode format, because of the format byte.
            serialize_with_format(value, format)
        }
        Ok(None) => {
            // This is how the currently released `bb` expects the data.
            bincode_serialize(value)
        }
        Err(e) => Err(std::io::Error::other(e)),
    }
}

#[cfg(test)]
mod tests {
    use acir_field::FieldElement;
    use brillig::{BitSize, HeapArray, IntegerBitSize, ValueOrArray};
    use std::str::FromStr;

    use crate::{
        circuit::{Opcode, brillig::BrilligFunctionId},
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
        use rmpv::Value;

        let value = ValueOrArray::HeapArray(HeapArray {
            pointer: brillig::MemoryAddress::Relative(0),
            size: 3,
        });
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap();

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
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap();

        assert_eq!(msg.as_str(), Some("U1"));
    }

    /// Test how an enum where some members are unit structs serializes.
    #[test]
    fn msgpack_repr_enum_of_mixed() {
        let value = vec![BitSize::Field, BitSize::Integer(IntegerBitSize::U64)];
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap();

        assert_eq!(format!("{msg}"), r#"["Field", {"Integer": "U64"}]"#);
    }

    /// Test that a newtype, just wrapping a value, is serialized as the underlying value.
    #[test]
    fn msgpack_repr_newtype() {
        use rmpv::Value;

        let value = Witness(1);
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap();

        assert!(matches!(msg, Value::Integer(_)));
    }

    /// Test to show that optional fields, when empty, are still in the map.
    /// The Rust library handles deserializing them as `None` if they are not present,
    /// but the `msgpack-c` library does not.
    #[test]
    fn msgpack_optional() {
        use rmpv::Value;

        let value: Opcode<FieldElement> = Opcode::BrilligCall {
            id: BrilligFunctionId(1),
            inputs: Vec::new(),
            outputs: Vec::new(),
            predicate: None,
        };
        let bz = msgpack_serialize(&value, false).unwrap();
        let msg = rmpv::decode::read_value::<&[u8]>(&mut bz.as_ref()).unwrap();

        let fields = msg.as_map().expect("enum is a map");
        let fields = &fields.first().expect("enum is non-empty").1;
        let fields = fields.as_map().expect("fields are map");

        let (k, v) = fields.last().expect("fields are not empty");
        assert_eq!(k.as_str().expect("names are str"), "predicate");
        assert!(matches!(v, Value::Nil));
    }

    #[test]
    fn format_from_str() {
        assert_eq!(Format::from_str("msgpack-compact").unwrap(), Format::MsgpackCompact);
    }
}
