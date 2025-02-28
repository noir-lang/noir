use color_eyre::eyre::{self, Context};

/// A protobuf codec to convert between a domain type `T`
/// and its protobuf representation `R`.
///
/// It is to be implemented on a `Self` independent of `T` and `R`,
/// so that `T` can be in a third party crate, and `Self` can be
/// generic in the `F` _field_ type as well, which would be cumbersome
/// if we had to implement traits on `R` because `T` is in another
/// crate from the schema, or to scatter the `.proto` schema around
/// so that the traits can be co-defined with `T` which is what can
/// actually be generic in `F`.
pub trait ProtoCodec<T, R> {
    /// Convert domain type `T` to protobuf representation `R`.
    fn encode(value: &T) -> R;
    /// Try to convert protobuf representation `R` to domain type `T`.
    fn decode(value: &R) -> eyre::Result<T>;
    /// Decode a field and attach the name of the field if it fails.
    fn decode_msg(value: &R, msg: &'static str) -> eyre::Result<T> {
        Self::decode(value).wrap_err(msg)
    }
    /// Encode multiple values as a vector.
    fn encode_vec<'a, I>(values: I) -> Vec<R>
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        values.into_iter().map(Self::encode).collect()
    }
    /// Decode multiple values into a vector.
    fn decode_vec(values: &[R]) -> eyre::Result<Vec<T>> {
        values.iter().map(Self::decode).collect()
    }
    /// Decode multiple values into a vector, attaching a field name to any errors.
    fn decode_vec_msg(values: &[R], msg: &'static str) -> eyre::Result<Vec<T>> {
        Self::decode_vec(values).wrap_err(msg)
    }
    /// Encode an optional `message` field as `Some`.
    fn encode_some(value: &T) -> Option<R> {
        Some(Self::encode(value))
    }
    /// Encode an `enum` to the `i32` value that `prost` represents it with.
    fn encode_enum(value: &T) -> i32
    where
        R: Into<i32>,
    {
        Self::encode(value).into()
    }
    /// Encode a domain type to protobuf and serialize it to bytes.
    fn serialize_to_vec(value: &T) -> Vec<u8>
    where
        R: prost::Message,
    {
        Self::encode(value).encode_to_vec()
    }
    /// Deserialize a buffer into protobuf and then decode into the domain type.
    fn deserialize_from_vec(buf: &[u8]) -> eyre::Result<T>
    where
        R: prost::Message + Default,
    {
        let repr = R::decode(buf).wrap_err("failed to decode into protobuf")?;
        Self::decode(&repr).wrap_err("failed to decode protobuf into domain")
    }
}
