use crate::meta::item_visibility::ItemVisibility;
use crate::option::Option;

/// A generic parameter of a [`TypeDefinition`].
///
/// It is returned by [`TypeDefinition::generics`].
///
/// Example:
///
/// ```noir
/// #[check]
/// pub struct Pair<T, let N: u32> {
///     pub items: [T; N],
///     count: u32,
/// }
///
/// comptime fn check(pair: TypeDefinition) {
///     let generics = pair.generics();
///     assert(generics[0].numeric_type().is_none());
///     assert_eq(generics[1].numeric_type().unwrap(), quote { u32 }.as_type());
/// }
/// ```
pub comptime struct TypeGeneric {
    typ: Type,
    numeric_type: Option<Type>,
}

impl TypeGeneric {
    /// The generic's type variable.
    ///
    /// It is the generic as written in the definition, such as `T`, not a concrete type.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let t = pair.generics()[0].typ();
    ///     assert_eq(f"{t}".quoted_contents(), quote { T });
    /// }
    /// ```
    pub comptime fn typ(self) -> Type {
        self.typ
    }

    /// The type of this generic if it is numeric, such as `u32` in `let N: u32`.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let generics = pair.generics();
    ///     // `T` is a type generic.
    ///     assert(generics[0].numeric_type().is_none());
    ///     // `N` is a numeric generic of type `u32`.
    ///     assert_eq(generics[1].numeric_type().unwrap(), quote { u32 }.as_type());
    /// }
    /// ```
    pub comptime fn numeric_type(self) -> Option<Type> {
        self.numeric_type
    }
}

/// A field of a [`TypeDefinition`].
///
/// It is returned by [`TypeDefinition::fields`] and [`TypeDefinition::fields_as_written`].
///
/// Example:
///
/// ```noir
/// #[check]
/// pub struct Pair<T, let N: u32> {
///     pub items: [T; N],
///     count: u32,
/// }
///
/// comptime fn check(pair: TypeDefinition) {
///     let items = pair.fields(@[quote { u8 }.as_type(), quote { 2 }.as_type()])[0];
///     assert_eq(items.name(), quote { items });
///     assert_eq(items.typ(), quote { [u8; 2] }.as_type());
///     assert(items.visibility().is_public());
/// }
/// ```
pub comptime struct StructField {
    name: Quoted,
    typ: Type,
    visibility: ItemVisibility,
}

impl StructField {
    /// The field's declared name, as a [`Quoted`] identifier.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     assert_eq(pair.fields_as_written()[1].name(), quote { count });
    /// }
    /// ```
    pub comptime fn name(self) -> Quoted {
        self.name
    }

    /// The field's declared type.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     assert_eq(pair.fields_as_written()[1].typ(), quote { u32 }.as_type());
    /// }
    /// ```
    pub comptime fn typ(self) -> Type {
        self.typ
    }

    /// The field's declared visibility.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let fields = pair.fields_as_written();
    ///     assert(fields[0].visibility().is_public());
    ///     assert(fields[1].visibility().is_private());
    /// }
    /// ```
    pub comptime fn visibility(self) -> ItemVisibility {
        self.visibility
    }
}

impl TypeDefinition {
    /// Adds an `#[abi(abi_argument)]` attribute to this type.
    ///
    /// The attribute exports the type in a contract's ABI under that tag, so it only has an effect
    /// on a type defined inside a `contract`.
    ///
    /// It fails for a type from another crate.
    ///
    /// Example:
    ///
    /// ```noir
    /// contract Token {
    ///     use std::meta::ctstring::AsCtString;
    ///
    ///     #[export_as_event]
    ///     pub struct Transfer {
    ///         amount: u64,
    ///     }
    ///
    ///     comptime fn export_as_event(transfer: TypeDefinition) {
    ///         transfer.add_abi("events".as_ctstring());
    ///         assert(transfer.has_builtin_attribute("abi"));
    ///     }
    /// }
    /// ```
    #[builtin(type_def_add_abi)]
    // docs:start:add_abi
    pub comptime fn add_abi(self, abi_argument: CtString) {}
    // docs:end:add_abi

    /// Returns this type definition as a type, with its generics as written: `Pair<T, N>` for
    /// `struct Pair<T, let N: u32>`.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let typ = pair.as_type();
    ///     assert_eq(f"{typ}".quoted_contents(), quote { Pair<T, N> });
    /// }
    /// ```
    #[builtin(type_def_as_type)]
    // docs:start:as_type
    pub comptime fn as_type(self) -> Type {}
    // docs:end:as_type

    /// Returns this type applied to the given generic arguments.
    ///
    /// Returns `Option::none()` if the number of generic arguments is wrong.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let generics = @[quote { u8 }.as_type(), quote { 2 }.as_type()];
    ///     assert_eq(pair.as_type_with_generics(generics).unwrap(), quote { Pair<u8, 2> }.as_type());
    ///
    ///     // `Pair` takes two generics.
    ///     assert(pair.as_type_with_generics(@[quote { u8 }.as_type()]).is_none());
    /// }
    /// ```
    #[builtin(type_def_as_type_with_generics)]
    // docs:start:as_type_with_generics
    pub comptime fn as_type_with_generics(self, generics: [Type]) -> Option<Type> {}
    // docs:end:as_type_with_generics

    /// Returns `true` if this type has an attribute called `name`.
    ///
    /// It matches built-in attributes, tags such as `#['my_tag]`, and comptime attribute macros. Use
    /// [`TypeDefinition::has_builtin_attribute`] to match only built-in ones.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #['my_tag]
    /// pub struct Point {
    ///     x: u32,
    /// }
    ///
    /// comptime fn check(point: TypeDefinition) {
    ///     assert(point.has_named_attribute("my_tag"));
    ///     assert(point.has_named_attribute("check"));
    ///     assert(!point.has_named_attribute("other"));
    /// }
    /// ```
    #[builtin(type_def_has_named_attribute)]
    // docs:start:has_named_attribute
    pub comptime fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_named_attribute

    /// Returns the arguments of each comptime attribute macro called `name` on this type.
    ///
    /// The outer vector has one entry per occurrence, so it is empty if the attribute isn't there. Each
    /// inner vector holds that occurrence's arguments as quoted expressions. Tags such as
    /// `#['my_tag(1)]` aren't macro calls, so they have no arguments here.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #[table(1)]
    /// pub struct Point {
    ///     x: u32,
    /// }
    ///
    /// comptime fn table(_point: TypeDefinition, _id: Field) {}
    ///
    /// comptime fn check(point: TypeDefinition) {
    ///     let tables = point.named_attribute_args("table");
    ///     assert_eq(tables.len(), 1);
    ///     assert_eq(tables[0][0], quote { 1 });
    /// }
    /// ```
    #[builtin(type_def_named_attribute_args)]
    // docs:start:named_attribute_args
    pub comptime fn named_attribute_args<let N: u32>(self, name: str<N>) -> [[Quoted]] {}
    // docs:end:named_attribute_args

    /// Returns `true` if this type has a built-in attribute called `name`, such as `allow` or `abi`.
    ///
    /// Unlike [`TypeDefinition::has_named_attribute`], it ignores tags and comptime attribute macros.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// #[allow(dead_code)]
    /// #['my_tag]
    /// pub struct Point {
    ///     x: u32,
    /// }
    ///
    /// comptime fn check(point: TypeDefinition) {
    ///     assert(point.has_builtin_attribute("allow"));
    ///     assert(!point.has_builtin_attribute("my_tag"));
    ///     assert(!point.has_builtin_attribute("check"));
    /// }
    /// ```
    #[builtin(type_def_has_builtin_attribute)]
    // docs:start:has_builtin_attribute
    pub comptime fn has_builtin_attribute<let N: u32>(self, name: str<N>) -> bool {}
    // docs:end:has_builtin_attribute

    /// Returns the source location of this type's name in its definition.
    ///
    /// Pass it to [`std::meta::error`](crate::meta::error) or [`std::meta::warn`](crate::meta::warn)
    /// to report a diagnostic at the type.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     // Warns "`Pair` is deprecated", pointing at `Pair`.
    ///     let name = pair.name();
    ///     std::meta::warn(f"`{name}` is deprecated", Option::<fmtstr<0, ()>>::none(), pair.location());
    /// }
    /// ```
    #[builtin(type_def_location)]
    // docs:start:location
    pub comptime fn location(self) -> Location {}
    // docs:end:location

    /// Returns each generic in this type definition.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let generics = pair.generics();
    ///     assert_eq(generics.len(), 2);
    ///     assert_eq(generics[1].numeric_type().unwrap(), quote { u32 }.as_type());
    /// }
    /// ```
    // docs:start:generics
    pub comptime fn generics(self) -> [TypeGeneric] {
        // docs:end:generics
        self.generics_raw().map(|g: (Type, Option<Type>)| {
            TypeGeneric { typ: g.0, numeric_type: g.1 }
        })
    }

    #[builtin(type_def_generics)]
    comptime fn generics_raw(self) -> [(Type, Option<Type>)] {}

    /// Returns each field in this type, with its generics replaced by `generic_args`.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let fields = pair.fields(@[quote { u8 }.as_type(), quote { 2 }.as_type()]);
    ///     assert_eq(fields[0].typ(), quote { [u8; 2] }.as_type());
    ///     assert_eq(fields[1].typ(), quote { u32 }.as_type());
    /// }
    /// ```
    // docs:start:fields
    pub comptime fn fields(self, generic_args: [Type]) -> [StructField] {
        // docs:end:fields
        self.fields_raw(generic_args).map(|f: (Quoted, Type, Quoted)| {
            StructField { name: f.0, typ: f.1, visibility: ItemVisibility::from_quoted(f.2) }
        })
    }

    #[builtin(type_def_fields)]
    comptime fn fields_raw(self, generic_args: [Type]) -> [(Quoted, Type, Quoted)] {}

    /// Returns each field in this type, with field types exactly as written.
    ///
    /// Field types may mention the type's own generics, such as `[T; N]`. Use
    /// [`TypeDefinition::fields`][Self::fields] when you need concrete types.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     let items = pair.fields_as_written()[0].typ();
    ///     assert_eq(f"{items}".quoted_contents(), quote { [T; N] });
    /// }
    /// ```
    // docs:start:fields_as_written
    pub comptime fn fields_as_written(self) -> [StructField] {
        // docs:end:fields_as_written
        self.fields_as_written_raw().map(|f: (Quoted, Type, Quoted)| {
            StructField { name: f.0, typ: f.1, visibility: ItemVisibility::from_quoted(f.2) }
        })
    }

    #[builtin(type_def_fields_as_written)]
    comptime fn fields_as_written_raw(self) -> [(Quoted, Type, Quoted)] {}

    /// Returns the module this type is defined in.
    ///
    /// Example:
    ///
    /// ```noir
    /// pub mod shapes {
    ///     #[check]
    ///     pub struct Square {
    ///         pub side: u32,
    ///     }
    ///
    ///     comptime fn check(square: TypeDefinition) {
    ///         assert_eq(square.module().name(), quote { shapes });
    ///     }
    /// }
    /// ```
    #[builtin(type_def_module)]
    // docs:start:module
    pub comptime fn module(self) -> Module {}
    // docs:end:module

    /// Returns the name of this type, without its path or generics.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub struct Pair<T, let N: u32> {
    ///     pub items: [T; N],
    ///     count: u32,
    /// }
    ///
    /// comptime fn check(pair: TypeDefinition) {
    ///     assert_eq(pair.name(), quote { Pair });
    /// }
    /// ```
    #[builtin(type_def_name)]
    // docs:start:name
    pub comptime fn name(self) -> Quoted {}
    // docs:end:name
}

impl crate::hash::Hash for TypeDefinition {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(type_def_hash(self))
    }
}

impl crate::cmp::Eq for TypeDefinition {
    #[builtin(type_def_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

#[builtin(type_def_hash)]
comptime fn type_def_hash(_type: TypeDefinition) -> Field {}
