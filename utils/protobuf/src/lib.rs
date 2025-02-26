use color_eyre::eyre;

/// Convert a type to/from Protobuf format.
pub trait ProtoCodec: Sized {
    /// The DTO type defined in the `.proto` schema representing `Self`.
    type Repr: prost::Message;
    /// Convert `Self` to `Repr`.
    fn encode(&self) -> Self::Repr;
    /// Try to convert `Repr` to `Self`.
    fn decode(value: &Self::Repr) -> eyre::Result<Self>;
}

/// Convert a Protobuf representation to/from a domain type.
/// We can use this to define protobuf mapping for types
/// defined in external crates, e.g. to place the `bar.proto`
/// definition in crate `foo`, and define the mappings from
/// `bar::proto::Bar` for `bar::Bar` in `foo::proto`.
pub trait ProtoRepr: Sized {
    /// The domain type.
    type Type;
    /// Convert `Type` to `Self`.
    fn encode(value: &Self::Type) -> Self;
    /// Try to convert `Self` to `Type`.
    fn decode(&self) -> eyre::Result<Self::Type>;
}

pub fn to_proto<T: ProtoCodec>(value: &T) -> T::Repr {
    value.encode()
}

pub fn into_proto<T: ProtoCodec>(value: T) -> T::Repr {
    to_proto(&value)
}

pub fn from_proto<T: ProtoCodec>(value: &T::Repr) -> eyre::Result<T> {
    T::decode(value)
}

pub fn to_proto_repr<R: ProtoRepr>(value: &R::Type) -> R {
    R::encode(value)
}

pub fn into_proto_repr<R: ProtoRepr>(value: R::Type) -> R {
    to_proto_repr(&value)
}

pub fn from_proto_repr<R: ProtoRepr>(value: &R) -> eyre::Result<R::Type> {
    value.decode()
}
