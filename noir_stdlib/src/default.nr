use crate::meta::derive_via;

/// Return an implementation-defined default value for the given type.
/// This is most often a zeroed value or an empty container, but there
/// are no actual restrictions on what an implementation could return.
#[derive_via(derive_default)]
// docs:start:default-trait
pub trait Default {
    fn default() -> Self;
}
// docs:end:default-trait

comptime fn derive_default(s: TypeDefinition) -> Quoted {
    let name = quote { $crate::default::Default };
    let signature = quote { fn default() -> Self };
    let for_each_field = |name| quote { $name: $crate::default::Default::default() };
    let body = |fields| quote { Self { $fields } };
    crate::meta::make_trait_impl(s, name, signature, for_each_field, quote { , }, body)
}

impl Default for Field {
    fn default() -> Field {
        0
    }
}

impl Default for u8 {
    fn default() -> u8 {
        0
    }
}
impl Default for u16 {
    fn default() -> u16 {
        0
    }
}
impl Default for u32 {
    fn default() -> u32 {
        0
    }
}
impl Default for u64 {
    fn default() -> u64 {
        0
    }
}

impl Default for u128 {
    fn default() -> u128 {
        0
    }
}

impl Default for i8 {
    fn default() -> i8 {
        0
    }
}
impl Default for i16 {
    fn default() -> i16 {
        0
    }
}
impl Default for i32 {
    fn default() -> i32 {
        0
    }
}
impl Default for i64 {
    fn default() -> i64 {
        0
    }
}

impl Default for () {
    fn default() -> () {
        ()
    }
}
impl Default for bool {
    fn default() -> bool {
        false
    }
}

impl<T, let N: u32> Default for [T; N]
where
    T: Default,
{
    fn default() -> [T; N] {
        [T::default(); N]
    }
}

impl<T> Default for [T] {
    fn default() -> [T] {
        [].as_vector()
    }
}

impl<A> Default for (A,)
where
    A: Default,
{
    fn default() -> (A,) {
        (A::default(),)
    }
}

impl<A, B> Default for (A, B)
where
    A: Default,
    B: Default,
{
    fn default() -> (A, B) {
        (A::default(), B::default())
    }
}

impl<A, B, C> Default for (A, B, C)
where
    A: Default,
    B: Default,
    C: Default,
{
    fn default() -> (A, B, C) {
        (A::default(), B::default(), C::default())
    }
}

impl<A, B, C, D> Default for (A, B, C, D)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
{
    fn default() -> (A, B, C, D) {
        (A::default(), B::default(), C::default(), D::default())
    }
}

impl<A, B, C, D, E> Default for (A, B, C, D, E)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
{
    fn default() -> (A, B, C, D, E) {
        (A::default(), B::default(), C::default(), D::default(), E::default())
    }
}

impl<A, B, C, D, E, F> Default for (A, B, C, D, E, F)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
    F: Default,
{
    fn default() -> (A, B, C, D, E, F) {
        (A::default(), B::default(), C::default(), D::default(), E::default(), F::default())
    }
}

impl<A, B, C, D, E, F, G> Default for (A, B, C, D, E, F, G)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
    F: Default,
    G: Default,
{
    fn default() -> (A, B, C, D, E, F, G) {
        (
            A::default(), B::default(), C::default(), D::default(), E::default(), F::default(),
            G::default(),
        )
    }
}

impl<A, B, C, D, E, F, G, H> Default for (A, B, C, D, E, F, G, H)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
    F: Default,
    G: Default,
    H: Default,
{
    fn default() -> (A, B, C, D, E, F, G, H) {
        (
            A::default(), B::default(), C::default(), D::default(), E::default(), F::default(),
            G::default(), H::default(),
        )
    }
}

impl<A, B, C, D, E, F, G, H, I> Default for (A, B, C, D, E, F, G, H, I)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
    F: Default,
    G: Default,
    H: Default,
    I: Default,
{
    fn default() -> (A, B, C, D, E, F, G, H, I) {
        (
            A::default(), B::default(), C::default(), D::default(), E::default(), F::default(),
            G::default(), H::default(), I::default(),
        )
    }
}

impl<A, B, C, D, E, F, G, H, I, J> Default for (A, B, C, D, E, F, G, H, I, J)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
    F: Default,
    G: Default,
    H: Default,
    I: Default,
    J: Default,
{
    fn default() -> (A, B, C, D, E, F, G, H, I, J) {
        (
            A::default(), B::default(), C::default(), D::default(), E::default(), F::default(),
            G::default(), H::default(), I::default(), J::default(),
        )
    }
}

impl<A, B, C, D, E, F, G, H, I, J, K> Default for (A, B, C, D, E, F, G, H, I, J, K)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
    F: Default,
    G: Default,
    H: Default,
    I: Default,
    J: Default,
    K: Default,
{
    fn default() -> (A, B, C, D, E, F, G, H, I, J, K) {
        (
            A::default(), B::default(), C::default(), D::default(), E::default(), F::default(),
            G::default(), H::default(), I::default(), J::default(), K::default(),
        )
    }
}

impl<A, B, C, D, E, F, G, H, I, J, K, L> Default for (A, B, C, D, E, F, G, H, I, J, K, L)
where
    A: Default,
    B: Default,
    C: Default,
    D: Default,
    E: Default,
    F: Default,
    G: Default,
    H: Default,
    I: Default,
    J: Default,
    K: Default,
    L: Default,
{
    fn default() -> (A, B, C, D, E, F, G, H, I, J, K, L) {
        (
            A::default(), B::default(), C::default(), D::default(), E::default(), F::default(),
            G::default(), H::default(), I::default(), J::default(), K::default(), L::default(),
        )
    }
}

mod tests {
    use super::Default;

    #[test]
    fn defaults() {
        assert_eq(<()>::default(), ());
        assert_eq(bool::default(), false);
        assert_eq(u8::default(), 0);
        assert_eq(u16::default(), 0);
        assert_eq(u32::default(), 0);
        assert_eq(u64::default(), 0);
        assert_eq(u128::default(), 0);
        assert_eq(i8::default(), 0);
        assert_eq(i16::default(), 0);
        assert_eq(i32::default(), 0);
        assert_eq(i64::default(), 0);
        assert_eq(Field::default(), 0);
        assert_eq(<[Field; 3]>::default(), [0; 3]);
        assert_eq(<[Field]>::default(), @[]);
        assert_eq(<(u8,)>::default(), (0,));
        assert_eq(<(u8, u8)>::default(), (0, 0));
        assert_eq(<(u8, u8, u8)>::default(), (0, 0, 0));
        assert_eq(<(u8, u8, u8, u8)>::default(), (0, 0, 0, 0));
        assert_eq(<(u8, u8, u8, u8, u8)>::default(), (0, 0, 0, 0, 0));
        assert_eq(<(u8, u8, u8, u8, u8, u8)>::default(), (0, 0, 0, 0, 0, 0));
        assert_eq(<(u8, u8, u8, u8, u8, u8, u8)>::default(), (0, 0, 0, 0, 0, 0, 0));
        assert_eq(<(u8, u8, u8, u8, u8, u8, u8, u8)>::default(), (0, 0, 0, 0, 0, 0, 0, 0));
        assert_eq(<(u8, u8, u8, u8, u8, u8, u8, u8, u8)>::default(), (0, 0, 0, 0, 0, 0, 0, 0, 0));
        assert_eq(
            <(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8)>::default(),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        );
        assert_eq(
            <(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8)>::default(),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        );
        assert_eq(
            <(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8)>::default(),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        );
    }
}
