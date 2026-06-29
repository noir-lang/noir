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
///
/// A type can also assert invariants of its own (ones that span several fields, or that the field
/// recursion cannot see) by naming one or more methods with `#[validate(Type::method)]`. The derived
/// `validate` calls each named method after recursing into the fields, so a custom invariant reads
/// fields that have already passed their own validation:
///
/// ```noir
/// #[derive(Validate)]
/// #[validate(Percent::in_range)]
/// struct Percent {
///     value: u8,
/// }
///
/// impl Percent {
///     fn in_range(&self) {
///         assert(self.value <= 100);
///     }
/// }
/// ```
///
/// Each named method must take a single `&self` argument and return nothing.
#[derive_via(derive_validate)]
// docs:start:validate-trait
pub trait Validate {
    fn validate(&self);
}
// docs:end:validate-trait

comptime fn derive_validate(s: TypeDefinition) -> Quoted {
    let signature = quote { fn validate(&self) };
    let for_each_field = |name| quote { self.$name.validate(); };

    // Splice a call to each `#[validate(Type::method)]` method after the field recursion, so a
    // custom invariant reads fields that have already passed their own validation rather than
    // unchecked data (e.g. a length field a method indexes by has already been bounds-checked).
    let mut custom = quote {};
    for occurrence in s.named_attribute_args("validate") {
        let method = occurrence[0];
        custom = quote { $custom $method(self); };
    }
    let body = |fields| quote { $fields $custom };

    crate::meta::make_trait_impl(
        s,
        quote { $crate::validate::Validate },
        signature,
        for_each_field,
        quote {},
        body,
    )
}

/// Attribute backing `#[validate(Type::method)]`. It does not generate any code itself — the
/// `#[derive(Validate)]` handler reads these attributes and splices the calls — but it checks that
/// each named method has the shape the derived `validate` expects to call: a single `self` argument
/// and no return value. Reporting that here points the error at the attribute rather than at the
/// generated impl.
pub comptime fn validate(s: TypeDefinition, method: FunctionDefinition) {
    let parameters = method.parameters();
    assert_eq(parameters.len(), 1, "a #[validate(...)] method must take a single `&self` argument");
    // The single argument must be `&self`: a reference to the annotated type. Comparing the
    // tokenized types rather than the `Type` values matches by the names a generic type prints
    // (e.g. `&BoundedVec<T, MaxLen>`), so the method's generics need not be the same type-variable
    // instances as the type definition's for the comparison to hold.
    let self_type = s.as_type();
    let parameter_type = parameters[0].1;
    let self_reference = quote { &$self_type }.as_type();
    assert_eq(
        quote { $parameter_type },
        quote { $self_reference },
        "a #[validate(...)] method must take `&self`",
    );
    assert_eq(
        method.return_type(),
        quote { () }.as_type(),
        "a #[validate(...)] method must not return a value",
    );
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
impl<A, B, C, D, E, F, G, H, I, J> Validate for (A, B, C, D, E, F, G, H, I, J)
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
    J: Validate,
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
        self.9.validate();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K> Validate for (A, B, C, D, E, F, G, H, I, J, K)
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
    J: Validate,
    K: Validate,
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
        self.9.validate();
        self.10.validate();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L> Validate for (A, B, C, D, E, F, G, H, I, J, K, L)
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
    J: Validate,
    K: Validate,
    L: Validate,
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
        self.9.validate();
        self.10.validate();
        self.11.validate();
    }
}
