use crate::meta::derive_via;

/// A type that asserts its own representation invariants.
///
/// `validate` should `assert` any invariant the type relies on but that the type system does not
/// itself guarantee — for example that a `BoundedVec`'s length does not exceed its capacity. It is
/// meant to be called on values that enter constrained code from an untrusted source (circuit
/// inputs, oracle results), where those invariants would otherwise be assumed without being checked.
///
/// `#[derive(Validate)]` implements it by validating each field in turn; the no-op implementations
/// below are the leaves of that recursion (primitive types carry no invariant of their own).
#[derive_via(derive_validate)]
pub trait Validate {
    fn validate(&self);
}

comptime fn derive_validate(s: TypeDefinition) -> Quoted {
    let signature = quote { fn validate(&self) };
    let for_each_field = |name| quote { self.$name.validate(); };
    let body = |fields| fields;
    crate::meta::make_trait_impl(
        s,
        quote { $crate::validate::Validate },
        signature,
        for_each_field,
        quote {},
        body,
    )
}

// Primitive leaves: their representation is their value, so there is nothing to assert.
impl Validate for Field {
    fn validate(&self) {}
}
impl Validate for u128 {
    fn validate(&self) {}
}
impl Validate for u64 {
    fn validate(&self) {}
}
impl Validate for u32 {
    fn validate(&self) {}
}
impl Validate for u16 {
    fn validate(&self) {}
}
impl Validate for u8 {
    fn validate(&self) {}
}
impl Validate for i8 {
    fn validate(&self) {}
}
impl Validate for i16 {
    fn validate(&self) {}
}
impl Validate for i32 {
    fn validate(&self) {}
}
impl Validate for i64 {
    fn validate(&self) {}
}
impl Validate for () {
    fn validate(&self) {}
}
impl Validate for bool {
    fn validate(&self) {}
}
impl<let N: u32> Validate for str<N> {
    fn validate(&self) {}
}

// Aggregates recurse into their elements.
impl<T, let N: u32> Validate for [T; N]
where
    T: Validate,
{
    fn validate(&self) {
        for i in 0..N {
            self[i].validate();
        }
    }
}

impl<T> Validate for [T]
where
    T: Validate,
{
    fn validate(&self) {
        for i in 0..self.len() {
            self[i].validate();
        }
    }
}

impl<A> Validate for (A,)
where
    A: Validate,
{
    fn validate(&self) {
        self.0.validate();
    }
}
impl<A, B> Validate for (A, B)
where
    A: Validate,
    B: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
    }
}
impl<A, B, C> Validate for (A, B, C)
where
    A: Validate,
    B: Validate,
    C: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
        self.2.validate();
    }
}
impl<A, B, C, D> Validate for (A, B, C, D)
where
    A: Validate,
    B: Validate,
    C: Validate,
    D: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
        self.2.validate();
        self.3.validate();
    }
}
impl<A, B, C, D, E> Validate for (A, B, C, D, E)
where
    A: Validate,
    B: Validate,
    C: Validate,
    D: Validate,
    E: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
        self.2.validate();
        self.3.validate();
        self.4.validate();
    }
}
impl<A, B, C, D, E, F> Validate for (A, B, C, D, E, F)
where
    A: Validate,
    B: Validate,
    C: Validate,
    D: Validate,
    E: Validate,
    F: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
        self.2.validate();
        self.3.validate();
        self.4.validate();
        self.5.validate();
    }
}
impl<A, B, C, D, E, F, G> Validate for (A, B, C, D, E, F, G)
where
    A: Validate,
    B: Validate,
    C: Validate,
    D: Validate,
    E: Validate,
    F: Validate,
    G: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
        self.2.validate();
        self.3.validate();
        self.4.validate();
        self.5.validate();
        self.6.validate();
    }
}
impl<A, B, C, D, E, F, G, H> Validate for (A, B, C, D, E, F, G, H)
where
    A: Validate,
    B: Validate,
    C: Validate,
    D: Validate,
    E: Validate,
    F: Validate,
    G: Validate,
    H: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
        self.2.validate();
        self.3.validate();
        self.4.validate();
        self.5.validate();
        self.6.validate();
        self.7.validate();
    }
}
impl<A, B, C, D, E, F, G, H, I> Validate for (A, B, C, D, E, F, G, H, I)
where
    A: Validate,
    B: Validate,
    C: Validate,
    D: Validate,
    E: Validate,
    F: Validate,
    G: Validate,
    H: Validate,
    I: Validate,
{
    fn validate(&self) {
        self.0.validate();
        self.1.validate();
        self.2.validate();
        self.3.validate();
        self.4.validate();
        self.5.validate();
        self.6.validate();
        self.7.validate();
        self.8.validate();
    }
}
