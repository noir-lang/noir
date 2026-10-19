use crate::cmp::Eq;
use crate::option::Option;

impl Quoted {
    /// Parses this token stream as an expression. Returns `Option::none()` if it doesn't parse.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo(1) }.as_expr().unwrap();
    ///     assert(call.as_function_call().is_some());
    ///
    ///     assert(quote { 1 + }.as_expr().is_none());
    /// }
    /// ```
    #[builtin(quoted_as_expr)]
    // docs:start:as_expr
    pub comptime fn as_expr(self) -> Option<Expr> {}
    // docs:end:as_expr

    /// Interprets this token stream as a path to a module. Returns `Option::none()` if it isn't a path
    /// or no module is found at it.
    ///
    /// Example:
    ///
    /// ```noir
    /// mod shapes {
    ///     pub mod circle {}
    /// }
    ///
    /// fn main() {
    ///     comptime {
    ///         let module = quote { shapes::circle }.as_module().unwrap();
    ///         assert_eq(module.name(), quote { circle });
    ///
    ///         assert(quote { shapes::square }.as_module().is_none());
    ///     }
    /// }
    /// ```
    #[builtin(quoted_as_module)]
    // docs:start:as_module
    pub comptime fn as_module(self) -> Option<Module> {}
    // docs:end:as_module

    /// Interprets this token stream as a trait constraint without its object type, such as `Eq` or
    /// `Into<Field>`.
    ///
    /// Compilation fails if the tokens don't parse as a trait constraint or name a trait that isn't in
    /// scope.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let constraint = quote { Eq }.as_trait_constraint();
    ///     assert(quote { u32 }.as_type().implements(constraint));
    /// }
    /// ```
    #[builtin(quoted_as_trait_constraint)]
    // docs:start:as_trait_constraint
    pub comptime fn as_trait_constraint(self) -> TraitConstraint {}
    // docs:end:as_trait_constraint

    /// Interprets this token stream as a resolved type.
    ///
    /// A numeric literal such as `4` is interpreted as the type-level integer `4`, so it can fill a
    /// numeric generic slot such as an array length.
    ///
    /// Compilation fails if the tokens don't parse as a type or name a type that isn't in scope.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let typ = quote { [u8; 4] }.as_type();
    ///     let array = typ.as_array().unwrap();
    ///     assert_eq(array.element_type(), quote { u8 }.as_type());
    ///     assert_eq(array.length_type(), quote { 4 }.as_type());
    /// }
    /// ```
    #[builtin(quoted_as_type)]
    // docs:start:as_type
    pub comptime fn as_type(self) -> Type {}
    // docs:end:as_type

    /// Returns the source location spanning the tokens of this stream, or `Option::none()` if it is empty.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     assert(quote { x + 1 }.location().is_some());
    ///     assert(quote {}.location().is_none());
    /// }
    /// ```
    #[builtin(quoted_location)]
    // docs:start:location
    pub comptime fn location(self) -> Option<Location> {}
    // docs:end:location

    /// Returns the individual tokens of this stream, each as its own `Quoted` value.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let tokens = quote { x + 1 }.tokens();
    ///     assert_eq(tokens.len(), 3);
    ///     assert_eq(tokens[1], quote { + });
    /// }
    /// ```
    #[builtin(quoted_tokens)]
    // docs:start:tokens
    pub comptime fn tokens(self) -> [Quoted] {}
    // docs:end:tokens
}

impl Eq for Quoted {
    #[builtin(quoted_eq)]
    comptime fn eq(self, other: Quoted) -> bool {}
}

impl crate::hash::Hash for Quoted {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(quoted_hash(self))
    }
}

#[builtin(quoted_hash)]
comptime fn quoted_hash(_quoted: Quoted) -> Field {}
