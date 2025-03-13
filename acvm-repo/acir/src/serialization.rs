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
#[allow(dead_code)]
pub(crate) fn msgpack_serialize<T: Serialize>(value: &T) -> std::io::Result<Vec<u8>> {
    // Could do this for the default behavior:
    // rmp_serde::to_vec(self).map_err(std::io::Error::other)

    // Or this to be able to configure the serialization:
    // * `Serializer::with_struct_map` encodes structs with field names instead of positions, which is backwards compatible when new fields are added, or optional fields removed.
    // * consider using `Serializer::with_bytes` to force buffers to be compact, or use `serde_bytes` on the field.
    // * enums have their name encoded in `Serializer::serialize_newtype_variant`, but originally it was done by index instead
    let mut buf = Vec::new();
    let mut ser = rmp_serde::Serializer::new(&mut buf).with_struct_map();
    value.serialize(&mut ser).map_err(std::io::Error::other)?;
    Ok(buf)
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
    use std::i32;

    use crate::serialization::{msgpack_deserialize, msgpack_serialize};

    /// Test that the MessagePack encoding we use is backwards for the cases we care about:
    /// * removal of an enum variant (e.g. opcode no longer in use)
    ///
    /// Things we could make backwards compatible but we don't necessarily want to:
    /// * struct fields added: the old reader could ignore new fields, but this would potentially lead to unintended behavior
    /// * struct fields reordered: this can be achieved if the previous point is acceptable
    /// * struct fields renamed: this would work with positional encoding, or using `#[serde(rename)]`
    #[test]
    fn msgpack_encodes_variant_name() {
        mod version1 {
            use serde::{Deserialize, Serialize};

            #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
            pub(crate) enum Foo {
                Bar { a: u64, b: String },
                Baz { c: i32 },
                Qux { d: Option<u32>, e: bool },
            }
        }

        mod version2 {
            use serde::{Deserialize, Serialize};

            // Removed `Bar` and `Qux::e`.
            #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
            pub(crate) enum Foo {
                Baz { c: i32 },
                Qux { e: bool },
            }
        }

        mod version3 {
            use serde::{Deserialize, Serialize};

            // Added `Baz::f` and `Qux::q`.
            #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
            pub(crate) enum Foo {
                Baz {
                    f: String,
                    c: i32,
                },
                Qux {
                    #[serde(rename = "e")]
                    h: bool,
                    g: String,
                },
            }
        }

        // If the enum encoding was position based, this would fail.
        let bz = msgpack_serialize(&version2::Foo::Baz { c: i32::MAX }).unwrap();
        let v = msgpack_deserialize::<version1::Foo>(&bz).expect("removed enum variant");
        assert_eq!(v, version1::Foo::Baz { c: i32::MAX });

        // If the struct field encoding was position based, this would fail.
        let bz =
            msgpack_serialize(&version3::Foo::Baz { f: "prefix".into(), c: i32::MAX }).unwrap();
        let v = msgpack_deserialize::<version1::Foo>(&bz)
            .expect("adding a new field before an existing one");
        assert_eq!(v, version1::Foo::Baz { c: i32::MAX });

        // If the struct field encoding was position based, this would fail.
        let bz = msgpack_serialize(&version3::Foo::Qux { h: true, g: "suffix".into() }).unwrap();
        let v = msgpack_deserialize::<version1::Foo>(&bz)
            .expect("adding a new field at the end and omit optional");
        assert_eq!(v, version1::Foo::Qux { d: None, e: true });
    }
}
