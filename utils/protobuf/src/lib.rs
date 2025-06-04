use color_eyre::eyre::{self, Context, bail, eyre};

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
    /// Encode a field as `Some`.
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
    /// Encode multiple values as a vector.
    fn encode_vec<'a, I>(values: I) -> Vec<R>
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        values.into_iter().map(Self::encode).collect()
    }

    /// Try to convert protobuf representation `R` to domain type `T`.
    fn decode(value: &R) -> eyre::Result<T>;
    /// Decode a field and attach the name of the field if it fails.
    fn decode_wrap(value: &R, msg: &'static str) -> eyre::Result<T> {
        Self::decode(value).wrap_err(msg)
    }
    /// Decode multiple values into a vector.
    fn decode_vec(values: &[R]) -> eyre::Result<Vec<T>> {
        values.iter().map(Self::decode).collect()
    }
    /// Decode multiple values into a vector, attaching a field name to any errors.
    fn decode_vec_wrap(values: &[R], msg: &'static str) -> eyre::Result<Vec<T>> {
        Self::decode_vec(values).wrap_err(msg)
    }
    /// Decode a fixed size array.
    fn decode_arr<const N: usize>(values: &[R]) -> eyre::Result<[T; N]> {
        match Self::decode_vec(values)?.try_into() {
            Ok(arr) => Ok(arr),
            Err(vec) => {
                bail!("expected {N} items, got {}", vec.len());
            }
        }
    }
    /// Decode a fixed size array, attaching a field name to any errors
    fn decode_arr_wrap<const N: usize>(values: &[R], msg: &'static str) -> eyre::Result<[T; N]> {
        Self::decode_arr(values).wrap_err(msg)
    }
    /// Decode a boxed fixed size array.
    fn decode_box_arr<const N: usize>(values: &[R]) -> eyre::Result<Box<[T; N]>> {
        Self::decode_arr(values).map(Box::new)
    }
    /// Decode a boxed fixed size array, attaching a field name to any errors
    fn decode_box_arr_wrap<const N: usize>(
        values: &[R],
        msg: &'static str,
    ) -> eyre::Result<Box<[T; N]>> {
        Self::decode_box_arr(values).wrap_err(msg)
    }
    /// Decode an optional field as a required one; fails if it's `None`.
    fn decode_some(value: &Option<R>) -> eyre::Result<T> {
        match value {
            Some(value) => Self::decode(value),
            None => Err(eyre!("missing field")),
        }
    }
    /// Decode an optional field as a required one, attaching a field name to any errors.
    /// Returns error if the field is missing.
    fn decode_some_wrap(value: &Option<R>, msg: &'static str) -> eyre::Result<T> {
        Self::decode_some(value).wrap_err(msg)
    }
    /// Decode an optional field, attaching a field name to any errors.
    /// Return `None` if the field is missing.
    fn decode_opt_wrap(value: &Option<R>, msg: &'static str) -> eyre::Result<Option<T>> {
        value.as_ref().map(|value| Self::decode_wrap(value, msg)).transpose()
    }
    /// Decode the numeric representation of an enum into the domain type.
    /// Return an error if the value cannot be recognized.
    fn decode_enum(value: i32) -> eyre::Result<T>
    where
        R: TryFrom<i32, Error = prost::UnknownEnumValue>,
    {
        let r = R::try_from(value)?;
        Self::decode(&r)
    }
    /// Decode the numeric representation of an enum, attaching the field name to any errors.
    fn decode_enum_wrap(value: i32, msg: &'static str) -> eyre::Result<T>
    where
        R: TryFrom<i32, Error = prost::UnknownEnumValue>,
    {
        Self::decode_enum(value).wrap_err(msg)
    }

    /// Encode a domain type to protobuf and serialize it to bytes.
    fn serialize_to_vec(value: &T) -> Vec<u8>
    where
        R: prost::Message,
    {
        Self::encode(value).encode_to_vec()
    }
    /// Deserialize a buffer into protobuf and then decode into the domain type.
    fn deserialize_from_slice(buf: &[u8]) -> eyre::Result<T>
    where
        R: prost::Message + Default,
    {
        let repr = R::decode(buf).wrap_err("failed to decode into protobuf")?;
        Self::decode(&repr).wrap_err("failed to decode protobuf into domain")
    }
}

/// Decode repeated items by mapping a function over them, attaching an error message if it fails.
/// Useful when a lambda needs to be applied before we can use one of the type class methods.
pub fn decode_vec_map_wrap<R, T, F>(rs: &[R], msg: &'static str, f: F) -> eyre::Result<Vec<T>>
where
    F: Fn(&R) -> eyre::Result<T>,
{
    rs.iter().map(f).collect::<eyre::Result<Vec<_>>>().wrap_err(msg)
}

/// Decode an optional item, returning an error if it's `None`.
/// Useful when a lambda needs to be applied before we can use one of the type class methods.
pub fn decode_some_map<R, T, F>(r: &Option<R>, f: F) -> eyre::Result<T>
where
    F: Fn(&R) -> eyre::Result<T>,
{
    match r {
        Some(r) => f(r),
        None => Err(eyre!("missing field")),
    }
}

/// Decode an optional item, attaching a field name to any errors.
/// Useful when a lambda needs to be applied before we can use one of the type class methods.
pub fn decode_some_map_wrap<R, T, F>(r: &Option<R>, msg: &'static str, f: F) -> eyre::Result<T>
where
    F: Fn(&R) -> eyre::Result<T>,
{
    decode_some_map(r, f).wrap_err(msg)
}

/// Decode a `oneof` field, returning an error if it's missing.
/// Useful when a lambda needs to be applied before we can use one of the type class methods.
pub fn decode_oneof_map<R, T, F>(r: &Option<R>, f: F) -> eyre::Result<T>
where
    F: Fn(&R) -> eyre::Result<T>,
{
    decode_some_map_wrap(r, "oneof value", f)
}
