//! Contains methods on the built-in `TypedExpr` type for resolved and type-checked expressions.
use crate::option::Option;

impl TypedExpr {
    /// If this expression refers to a function definitions, returns it. Otherwise returns `Option::none()`.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn double(x: u32) -> u32 {
    ///     x * 2
    /// }
    ///
    /// fn main() {
    ///     assert_eq(double(3), 6);
    ///
    ///     comptime {
    ///         let function = quote { double }.as_expr().unwrap().resolve(Option::none());
    ///         assert_eq(function.as_function_definition().unwrap().name(), quote { double });
    ///
    ///         let call = quote { double(1) }.as_expr().unwrap().resolve(Option::none());
    ///         assert(call.as_function_definition().is_none());
    ///     }
    /// }
    /// ```
    #[builtin(typed_expr_as_function_definition)]
    // docs:start:as_function_definition
    pub comptime fn as_function_definition(self) -> Option<FunctionDefinition> {}
    // docs:end:as_function_definition

    /// Returns the type of the expression, if the expression could be resolved without errors.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let typed = quote { 1_u8 + 2 }.as_expr().unwrap().resolve(Option::none());
    ///     assert_eq(typed.get_type().unwrap(), quote { u8 }.as_type());
    /// }
    /// ```
    #[builtin(typed_expr_get_type)]
    // docs:start:get_type
    pub comptime fn get_type(self) -> Option<Type> {}
    // docs:end:get_type

    /// Returns the source location of this expression.
    ///
    /// An `Expr` doesn't keep the location of the tokens it was parsed from, so for a `TypedExpr`
    /// returned by [`Expr::resolve`] this is the location of the receiver of that call: `expr` in
    /// `expr.resolve(...)`.
    ///
    /// Pass it to [`std::meta::error`](crate::meta::error) or [`std::meta::warn`](crate::meta::warn)
    /// to report a diagnostic there.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let expr = quote { 1 + 2 }.as_expr().unwrap();
    ///     let first = expr.resolve(Option::none());
    ///     let second = expr.resolve(Option::none());
    ///     // Each `resolve` call is on its own line, so each result has its own location.
    ///     assert(first.location() != second.location());
    /// }
    /// ```
    #[builtin(typed_expr_location)]
    // docs:start:location
    pub comptime fn location(self) -> Location {}
    // docs:end:location
}
