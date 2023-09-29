mod bool_serialization;
mod field_serialization;
mod u32_serialization;
mod aztec_address_serialization;

/**
 * Before Noir supports traits, a way of specifying the serialization and deserialization methods for a type.
 */
// docs:start:TypeSerializationInterface
struct TypeSerializationInterface<T, N> {
    deserialize: fn ([Field; N]) -> T,
    serialize: fn (T) -> [Field; N],
}
// docs:end:TypeSerializationInterface