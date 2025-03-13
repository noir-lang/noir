//! Serialization formats we consider using for the bytecode and the witness stack.
use crate::proto::convert::ProtoSchema;
use acir_field::AcirField;
use noir_protobuf::ProtoCodec;
use serde::{Deserialize, Serialize};

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
        let mut buf = Vec::new();
        let mut ser = rmp_serde::Serializer::new(&mut buf).with_struct_map();
        value.serialize(&mut ser).map_err(std::io::Error::other)?;
        Ok(buf)
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

#[cfg(test)]
mod tests {
    use crate::serialization::{msgpack_deserialize, msgpack_serialize};

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
}
