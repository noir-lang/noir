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
    // * by default structs would become tuples, which isn't backwards compatible
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
