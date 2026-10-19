use crate::option::Option;

impl Module {
    /// Returns `true` if this module has an attribute called `name`.
    ///
    /// It matches built-in attributes, tags such as `#['my_tag]`, and comptime attribute macros. Use
    /// [`Module::has_builtin_attribute`] to match only built-in ones.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #['my_tag]
    /// mod shapes {}
    ///
    /// comptime fn check(shapes: Module) {
    ///     assert(shapes.has_named_attribute("my_tag"));
    ///     assert(shapes.has_named_attribute("check"));
    ///     assert(!shapes.has_named_attribute("other"));
    /// }
    /// ```
    #[builtin(module_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    /// Returns the arguments of each comptime attribute macro called `name` on this module.
    ///
    /// The outer vector has one entry per occurrence, so it is empty if the attribute isn't there. Each
    /// inner vector holds that occurrence's arguments as quoted expressions. Tags such as
    /// `#['my_tag(1)]` aren't macro calls, so they have no arguments here.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #[version(2)]
    /// mod shapes {}
    ///
    /// comptime fn version(_module: Module, _version: Field) {}
    ///
    /// comptime fn check(shapes: Module) {
    ///     let versions = shapes.named_attribute_args("version");
    ///     assert_eq(versions.len(), 1);
    ///     assert_eq(versions[0][0], quote { 2 });
    /// }
    /// ```
    #[builtin(module_named_attribute_args)]
    // docs:start:named_attribute_args
    pub comptime fn named_attribute_args<let N: u32>(self, name: str<N>) -> [[Quoted]] {}
    // docs:end:named_attribute_args

    /// Returns `true` if this module has a built-in attribute called `name`.
    ///
    /// Unlike [`Module::has_named_attribute`], it ignores tags and comptime attribute macros.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #['my_tag]
    /// mod shapes {}
    ///
    /// comptime fn check(shapes: Module) {
    ///     assert(!shapes.has_builtin_attribute("my_tag"));
    ///     assert(!shapes.has_builtin_attribute("check"));
    /// }
    /// ```
    #[builtin(module_has_builtin_attribute)]
    // docs:start:has_builtin_attribute
    pub comptime fn has_builtin_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_builtin_attribute

    /// Returns `true` if this module was declared with `contract name { ... }` rather than `mod`.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// mod shapes {
    ///     pub fn area(width: u32, height: u32) -> u32 {
    ///         width * height
    ///     }
    ///
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     pub mod circle {}
    /// }
    ///
    /// comptime fn check(shapes: Module) {
    ///     assert(!shapes.is_contract());
    /// }
    /// ```
    #[builtin(module_is_contract)]
    // docs:start:is_contract
    pub comptime fn is_contract(self) -> bool {}
    // docs:end:is_contract

    /// Returns the source location of this module's name in its declaration.
    ///
    /// Pass it to [`std::meta::error`](crate::meta::error) or [`std::meta::warn`](crate::meta::warn)
    /// to report a diagnostic at the module.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// mod shapes {
    ///     pub fn area(width: u32, height: u32) -> u32 {
    ///         width * height
    ///     }
    ///
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     pub mod circle {}
    /// }
    ///
    /// comptime fn check(shapes: Module) {
    ///     // Warns "`shapes` is deprecated", pointing at `shapes`.
    ///     let name = shapes.name();
    ///     std::meta::warn(f"`{name}` is deprecated", Option::<fmtstr<0, ()>>::none(), shapes.location());
    /// }
    /// ```
    #[builtin(module_location)]
    // docs:start:location
    pub comptime fn location(self) -> Location {}
    // docs:end:location

    /// Returns each function defined directly in this module.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// mod shapes {
    ///     pub fn area(width: u32, height: u32) -> u32 {
    ///         width * height
    ///     }
    ///
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     pub mod circle {}
    /// }
    ///
    /// comptime fn check(shapes: Module) {
    ///     let functions = shapes.functions();
    ///     assert_eq(functions.len(), 1);
    ///     assert_eq(functions[0].name(), quote { area });
    /// }
    /// ```
    #[builtin(module_functions)]
    // docs:start:functions
    pub comptime fn functions(self) -> [FunctionDefinition] {}
    // docs:end:functions

    /// Returns each struct defined directly in this module.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// mod shapes {
    ///     pub fn area(width: u32, height: u32) -> u32 {
    ///         width * height
    ///     }
    ///
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     pub mod circle {}
    /// }
    ///
    /// comptime fn check(shapes: Module) {
    ///     let structs = shapes.structs();
    ///     assert_eq(structs.len(), 1);
    ///     assert_eq(structs[0].name(), quote { Square });
    /// }
    /// ```
    #[builtin(module_structs)]
    // docs:start:structs
    pub comptime fn structs(self) -> [TypeDefinition] {}
    // docs:end:structs

    /// Returns each module declared directly inside this module.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// mod shapes {
    ///     pub fn area(width: u32, height: u32) -> u32 {
    ///         width * height
    ///     }
    ///
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     pub mod circle {}
    /// }
    ///
    /// comptime fn check(shapes: Module) {
    ///     let children = shapes.child_modules();
    ///     assert_eq(children.len(), 1);
    ///     assert_eq(children[0].name(), quote { circle });
    /// }
    /// ```
    #[builtin(module_child_modules)]
    // docs:start:child_modules
    pub comptime fn child_modules(self) -> [Module] {}
    // docs:end:child_modules

    /// Returns the name of this module, without its path.
    ///
    /// A crate's root module has no name, so it returns an empty `Quoted`.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// mod shapes {
    ///     pub fn area(width: u32, height: u32) -> u32 {
    ///         width * height
    ///     }
    ///
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     pub mod circle {}
    /// }
    ///
    /// comptime fn check(shapes: Module) {
    ///     assert_eq(shapes.name(), quote { shapes });
    ///     assert_eq(shapes.child_modules()[0].name(), quote { circle });
    /// }
    /// ```
    #[builtin(module_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name

    /// Returns the module this module is declared in, or `Option::none()` for a crate's root module.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// mod shapes {
    ///     pub fn area(width: u32, height: u32) -> u32 {
    ///         width * height
    ///     }
    ///
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     pub mod circle {}
    /// }
    ///
    /// comptime fn check(shapes: Module) {
    ///     let circle = shapes.child_modules()[0];
    ///     assert_eq(circle.parent().unwrap(), shapes);
    /// }
    /// ```
    #[builtin(module_parent)]
    // docs:start:parent
    pub comptime fn parent(self) -> Option<Module> {}
    // docs:end:parent
}

impl crate::hash::Hash for Module {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(module_hash(self))
    }
}

impl crate::cmp::Eq for Module {
    #[builtin(module_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

#[builtin(module_hash)]
comptime fn module_hash(_module: Module) -> Field {}
