impl TraitImpl {
    /// Returns the generic arguments of the trait this impl implements.
    ///
    /// For `impl Convert<Field> for Meters`, that is `[Field]`.
    ///
    /// Example:
    ///
    /// ```noir
    /// pub trait Convert<T> {
    ///     fn convert(self) -> T;
    /// }
    ///
    /// pub struct Meters {
    ///     value: u32,
    /// }
    ///
    /// impl Convert<Field> for Meters {
    ///     fn convert(self) -> Field {
    ///         self.value as Field
    ///     }
    /// }
    ///
    /// fn main() {
    ///     comptime {
    ///         let meters = quote { Meters }.as_type();
    ///         let constraint = quote { Convert<Field> }.as_trait_constraint();
    ///         let the_impl = meters.get_trait_impl(constraint).unwrap();
    ///         assert_eq(the_impl.trait_generic_args(), @[quote { Field }.as_type()]);
    ///     }
    /// }
    /// ```
    #[builtin(trait_impl_trait_generic_args)]
    // docs:start:trait_generic_args
    pub comptime fn trait_generic_args(self) -> [Type] {}
    // docs:end:trait_generic_args

    /// Returns each method defined in this impl.
    ///
    /// Example:
    ///
    /// ```noir
    /// pub trait Convert<T> {
    ///     fn convert(self) -> T;
    /// }
    ///
    /// pub struct Meters {
    ///     value: u32,
    /// }
    ///
    /// impl Convert<Field> for Meters {
    ///     fn convert(self) -> Field {
    ///         self.value as Field
    ///     }
    /// }
    ///
    /// fn main() {
    ///     comptime {
    ///         let meters = quote { Meters }.as_type();
    ///         let constraint = quote { Convert<Field> }.as_trait_constraint();
    ///         let the_impl = meters.get_trait_impl(constraint).unwrap();
    ///         let methods = the_impl.methods();
    ///         assert_eq(methods.len(), 1);
    ///         assert_eq(methods[0].name(), quote { convert });
    ///     }
    /// }
    /// ```
    #[builtin(trait_impl_methods)]
    // docs:start:methods
    pub comptime fn methods(self) -> [FunctionDefinition] {}
    // docs:end:methods
}
