mod field_value;

struct PublicStateValueInterface<T, N> {
    deserialise: fn ([Field; N]) -> T,
    serialise: fn (T) -> [Field; N],
}