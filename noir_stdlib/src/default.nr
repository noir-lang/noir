use crate::meta::derive_via;

#[derive_via(derive_default)]
// docs:start:default-trait
pub trait Default {
    fn default() -> Self;
}
// docs:end:default-trait

comptime fn derive_default(s: StructDefinition) -> Quoted {
    let name = quote { Default };
    let signature = quote { fn default() -> Self };
    let for_each_field = |name| quote { $name: Default::default() };
    let body = |fields| quote { Self { $fields } };
    crate::meta::make_trait_impl(s, name, signature, for_each_field, quote { , }, body)
}

impl Default for Field {
    fn default() -> Field {
        0
    }
}

impl Default for u1 {
    fn default() -> u1 {
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
        &[]
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
