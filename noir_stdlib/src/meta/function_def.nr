use crate::meta::item_visibility::ItemVisibility;

/// A single parameter of a [`FunctionDefinition`].
///
/// It is returned by [`FunctionDefinition::parameters`].
///
/// Example:
///
/// ```noir
/// #[check]
/// pub fn add(x: u32, y: u32) -> u32 {
///     x + y
/// }
///
/// comptime fn check(f: FunctionDefinition) {
///     let x = f.parameters()[0];
///     assert_eq(x.name(), quote { x });
///     assert_eq(x.typ(), quote { u32 }.as_type());
/// }
/// ```
pub comptime struct FunctionParameter {
    name: Quoted,
    typ: Type,
}

impl FunctionParameter {
    /// The parameter's declared name, as a [`Quoted`] identifier.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert_eq(f.parameters()[1].name(), quote { y });
    /// }
    /// ```
    pub comptime fn name(self) -> Quoted {
        self.name
    }

    /// The parameter's declared type.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert_eq(f.parameters()[1].typ(), quote { u32 }.as_type());
    /// }
    /// ```
    pub comptime fn typ(self) -> Type {
        self.typ
    }
}

impl FunctionDefinition {
    /// Returns this function as a [`TypedExpr`].
    ///
    /// Splice it into quoted code to refer to the function.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     let function = f.as_typed_expr();
    ///     let call = quote { $function(1, 2) };
    ///     assert_eq(call, quote { add(1, 2) });
    /// }
    /// ```
    #[builtin(function_def_as_typed_expr)]
    // docs:start:as_typed_expr
    pub comptime fn as_typed_expr(self) -> TypedExpr {}
    // docs:end:as_typed_expr

    /// Returns the body of this function as a block expression.
    ///
    /// Only functions of the current crate that haven't been resolved yet have a body to return, so it
    /// fails for a function that was already called at compile time or that comes from another
    /// crate.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     let statements = f.body().as_block().unwrap();
    ///     assert_eq(statements.len(), 1);
    ///     assert(statements[0].as_binary_op().unwrap().operator().is_add());
    /// }
    /// ```
    #[builtin(function_def_body)]
    // docs:start:body
    pub comptime fn body(self) -> Expr {}
    // docs:end:body

    /// Makes any call to this function a compile error with `error_message`.
    ///
    /// An attribute that generates a replacement for a function can use it to point callers to the
    /// replacement. A disabled function in a contract is also removed from the contract's interface,
    /// as if it had `#[contract_library_method]`.
    ///
    /// It must run before the function is resolved: it fails if the function was already called at
    /// compile time or comes from another crate.
    ///
    /// # Security
    ///
    /// An attribute macro from a dependency can disable any function it can name, including a
    /// contract entry point. The program still compiles as long as nothing calls the function, so test
    /// every entry point and review the output of `nargo expand` for dependencies you don't trust.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::meta::ctstring::AsCtString;
    ///
    /// #[replaced_by_new]
    /// pub fn old() {}
    ///
    /// comptime fn replaced_by_new(f: FunctionDefinition) {
    ///     f.disable(f"`old` is disabled, use `new` instead".as_ctstring());
    /// }
    ///
    /// // Calling `old()` anywhere is now a compile error: "`old` is disabled, use `new` instead".
    /// ```
    #[builtin(function_def_disable)]
    // docs:start:disable
    pub comptime fn disable(self, error_message: CtString) {}
    // docs:end:disable

    /// Returns `true` if this function has an attribute called `name`.
    ///
    /// It matches built-in attributes such as `deprecated`, tags such as `#['my_tag]`, and comptime
    /// attribute macros. Use [`FunctionDefinition::has_builtin_attribute`] to match only built-in ones.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #['my_tag]
    /// #[deprecated]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert(f.has_named_attribute("my_tag"));
    ///     assert(f.has_named_attribute("deprecated"));
    ///     assert(f.has_named_attribute("check"));
    ///     assert(!f.has_named_attribute("inline"));
    /// }
    /// ```
    #[builtin(function_def_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    /// Returns the arguments of each comptime attribute macro called `name` on this function.
    ///
    /// The outer vector has one entry per occurrence, so it is empty if the attribute isn't there. Each
    /// inner vector holds that occurrence's arguments as quoted expressions. Tags such as
    /// `#['my_tag(1)]` aren't macro calls, so they have no arguments here.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #[route(1)]
    /// #[route(2 + 3)]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn route(_f: FunctionDefinition, _id: Field) {}
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     let routes = f.named_attribute_args("route");
    ///     assert_eq(routes.len(), 2);
    ///     assert_eq(routes[0][0], quote { 1 });
    ///     assert_eq(routes[1][0], quote { (2 + 3) });
    ///
    ///     assert_eq(f.named_attribute_args("missing").len(), 0);
    /// }
    /// ```
    #[builtin(function_def_named_attribute_args)]
    // docs:start:named_attribute_args
    pub comptime fn named_attribute_args<let N: u32>(self, name: str<N>) -> [[Quoted]] {}
    // docs:end:named_attribute_args

    /// Returns `true` if this function has a built-in attribute called `name`, such as `deprecated`.
    ///
    /// Unlike [`FunctionDefinition::has_named_attribute`], it ignores tags and comptime attribute
    /// macros, so a user attribute that shares a built-in's name doesn't match.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #[deprecated]
    /// #['my_tag]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert(f.has_builtin_attribute("deprecated"));
    ///     assert(!f.has_builtin_attribute("my_tag"));
    ///     assert(!f.has_builtin_attribute("check"));
    /// }
    /// ```
    #[builtin(function_def_has_builtin_attribute)]
    // docs:start:has_builtin_attribute
    pub comptime fn has_builtin_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_builtin_attribute

    /// Returns `true` if this function is `unconstrained`.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub unconstrained fn hint(x: u32) -> u32 {
    ///     x / 2
    /// }
    ///
    /// #[check_add]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert(f.is_unconstrained());
    /// }
    ///
    /// comptime fn check_add(f: FunctionDefinition) {
    ///     assert(!f.is_unconstrained());
    /// }
    /// ```
    #[builtin(function_def_is_unconstrained)]
    // docs:start:is_unconstrained
    pub comptime fn is_unconstrained(self) -> bool {}
    // docs:end:is_unconstrained

    /// Returns the source location of this function's name.
    ///
    /// Pass it to [`std::meta::error`](crate::meta::error) or [`std::meta::warn`](crate::meta::warn)
    /// to report a diagnostic at the function.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     // Warns "`add` overflows for large inputs", pointing at `add`.
    ///     let name = f.name();
    ///     std::meta::warn(f"`{name}` overflows for large inputs", Option::<fmtstr<0, ()>>::none(), f.location());
    /// }
    /// ```
    #[builtin(function_def_location)]
    // docs:start:location
    pub comptime fn location(self) -> Location {}
    // docs:end:location

    /// Returns the module this function is defined in.
    ///
    /// Example:
    ///
    /// ```noir
    /// pub mod math {
    ///     #[check]
    ///     pub fn add(x: u32, y: u32) -> u32 {
    ///         x + y
    ///     }
    ///
    ///     comptime fn check(f: FunctionDefinition) {
    ///         assert_eq(f.module().name(), quote { math });
    ///     }
    /// }
    /// ```
    #[builtin(function_def_module)]
    // docs:start:module
    pub comptime fn module(self) -> Module {}
    // docs:end:module

    /// Returns the name of this function, without its path or generics.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert_eq(f.name(), quote { add });
    /// }
    /// ```
    #[builtin(function_def_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name

    /// Returns the parameters this function was declared with, in source order.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     let parameters = f.parameters();
    ///     assert_eq(parameters.len(), 2);
    ///     assert_eq(parameters[0].name(), quote { x });
    ///     assert_eq(parameters[1].typ(), quote { u32 }.as_type());
    /// }
    /// ```
    // docs:start:parameters
    pub comptime fn parameters(self) -> [FunctionParameter] {
        // docs:end:parameters
        self.parameters_raw().map(|param: (Quoted, Type)| {
            FunctionParameter { name: param.0, typ: param.1 }
        })
    }

    #[builtin(function_def_parameters)]
    comptime fn parameters_raw(self) -> [(Quoted, Type)] {}

    /// Returns the return type of this function. It is the unit type `()` if it declares none.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// #[check_log]
    /// pub fn log(x: u32) {
    ///     println(x);
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert_eq(f.return_type(), quote { u32 }.as_type());
    /// }
    ///
    /// comptime fn check_log(f: FunctionDefinition) {
    ///     assert(f.return_type().is_unit());
    /// }
    /// ```
    #[builtin(function_def_return_type)]
    // docs:start:return_type
    pub comptime fn return_type(self) -> Type {}
    // docs:end:return_type

    /// Returns the function's visibility.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub(crate) fn add(x: u32, y: u32) -> u32 {
    ///     x + y
    /// }
    ///
    /// comptime fn check(f: FunctionDefinition) {
    ///     assert(f.visibility().is_public_crate());
    /// }
    /// ```
    // docs:start:visibility
    pub comptime fn visibility(self) -> ItemVisibility {
        // docs:end:visibility
        ItemVisibility::from_quoted(self.visibility_raw())
    }

    #[builtin(function_def_visibility)]
    comptime fn visibility_raw(self) -> Quoted {}
}

impl crate::hash::Hash for FunctionDefinition {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(function_def_hash(self))
    }
}

impl crate::cmp::Eq for FunctionDefinition {
    #[builtin(function_def_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

#[builtin(function_def_hash)]
comptime fn function_def_hash(_function: FunctionDefinition) -> Field {}
