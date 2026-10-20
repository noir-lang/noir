/// A fixed-size array, denoted `[T; N]`, for the element type, `T`,
/// and the non-negative compile-time constant size, `N`.
///
/// There are two syntactic forms for creating an array:
/// * A vector with each element, i.e., `[x, y, z]`.
/// * A repeat expression `[expr; N]` where `N` is how many times to repeat `expr` in the array.
#['nargo_doc_primitive array]
mod primitive_array {}

/// The boolean type.
///
/// The `bool` represents a value, which could only be either `true` or `false`.
/// If you cast a `bool` into an integer, `true` will be `1` and `false` will be `0`.
#['nargo_doc_primitive bool]
mod primitive_bool {}

/// The 8-bit unsigned integer type.
#['nargo_doc_primitive u8]
mod primitive_u8 {}

/// The 16-bit unsigned integer type.
#['nargo_doc_primitive u16]
mod primitive_u16 {}

/// The 32-bit unsigned integer type.
#['nargo_doc_primitive u32]
mod primitive_u32 {}

/// The 64-bit unsigned integer type.
#['nargo_doc_primitive u64]
mod primitive_u64 {}

/// The 128-bit unsigned integer type.
#['nargo_doc_primitive u128]
mod primitive_u128 {}

/// The 8-bit signed integer type.
#['nargo_doc_primitive i8]
mod primitive_i8 {}

/// The 16-bit signed integer type.
#['nargo_doc_primitive i16]
mod primitive_i16 {}

/// The 32-bit signed integer type.
#['nargo_doc_primitive i32]
mod primitive_i32 {}

/// The 64-bit signed integer type.
#['nargo_doc_primitive i64]
mod primitive_i64 {}

/// The native field type of the proving backend.
///
/// The size of a Noir field depends on the elliptic curve's finite field for the proving backend adopted.
/// For example, a field would be a 254-bit integer when paired with the default backend that spans the Grumpkin curve.
///
/// Fields support integer arithmetic.
///
/// If proving efficiency is of priority, fields should be used as a default for solving problems.
/// Smaller integer types (e.g. `u64`) incur extra range constraints.
#['nargo_doc_primitive Field]
mod primitive_Field {}

/// A a compile-time, dynamically-sized string type.
///
/// Compared to [`str`]`<N>` and [`fmtstr`]`<N, T>`,
/// [`CtString`] is useful because its size does not need to be specified in its type. This
/// can be used for formatting items at compile-time or general string handling in `comptime`
/// code.
///
/// Since [`fmtstr`]s can be converted into `CtString`s, you can make use of their formatting
/// abilities in CtStrings by formatting in [`fmtstr`]s then converting the result to a CtString
/// afterward.
#['nargo_doc_primitive CtString]
mod primitive_CtString {}

/// A compile-time, quoted, syntactically valid expression.
///
/// An [`Expr`] can be obtained using [`Quoted::as_expr`]:
///
/// ```noir
/// fn main() {
///     comptime {
///         let expr = quote { 1 + 2 }.as_expr().unwrap();
///         assert(expr.as_binary_op().unwrap().operator().is_add());
///     }
/// }
/// ```
#['nargo_doc_primitive Expr]
mod primitive_Expr {}

/// A formatted string with interpolated values.
///
/// # Example
///
/// ```noir
/// use std::meta::ctstring::AsCtString;
///
/// fn main() {
///     let x = 1;
///     let y = 2;
///     let sum = 3;
///     // A `Field` is formatted in hexadecimal.
///     println(f"{x} + {y} is {sum}"); // prints "0x01 + 0x02 is 0x03"
///
///     comptime {
///         let a = 1_u32;
///         let b = 2_u32;
///         let total = a + b;
///         assert_eq(f"{a} + {b} is {total}".as_ctstring(), "1 + 2 is 3".as_ctstring());
///     }
/// }
/// ```
#['nargo_doc_primitive fmtstr]
mod primitive_fmtstr {}

/// A compile-time type which represents a function definition in this source program.
///
/// A [`FunctionDefinition`] is passed as the first argument to a macro attribute on
/// top of a function:
///
/// ```noir
/// #[attr]
/// pub fn foo() {}
///
/// comptime fn attr(function: FunctionDefinition) {
///     assert_eq(function.name(), quote { foo });
/// }
/// ```
#['nargo_doc_primitive FunctionDefinition]
mod primitive_FunctionDefinition {}

/// A compile-time type which represents a source location in this program.
///
/// `FunctionDefinition`, `Module`, `TraitDefinition`, `TypeDefinition` and `TypedExpr` each have a
/// `location` method. Pass the result to [`std::meta::error`](crate::meta::error) or
/// [`std::meta::warn`](crate::meta::warn) to report a diagnostic at that item or expression.
///
/// Two locations are equal when they refer to the same source span.
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
///     let location: Location = f.location();
///     assert_eq(location, f.location());
///     assert(location != f.module().location());
/// }
/// ```
#['nargo_doc_primitive Location]
mod primitive_Location {}

/// A compile-time type which represents a module in this source program.
///
/// A [`Module`] is passed as the first argument to a macro attribute on top of a module:
///
/// ```noir
/// #[attr]
/// mod foo {}
///
/// comptime fn attr(module: Module) {
///     assert_eq(module.name(), quote { foo });
/// }
/// ```
///
/// It can also be obtained using [`Quoted::as_module`]:
///
/// ```noir
/// mod foo {}
///
/// fn main() {
///     comptime {
///         let module = quote { foo }.as_module().unwrap();
///         assert_eq(module.name(), quote { foo });
///     }
/// }
/// ```
#['nargo_doc_primitive Module]
mod primitive_Module {}

/// A compile-time type which represents quoted token streams and is the result of the
/// `quote { ... }` expression.
#['nargo_doc_primitive Quoted]
mod primitive_Quoted {}

/// A dynamically-sized view into a contiguous sequence, `[T]`.
///
/// There are two syntactic forms for creating a vector:
/// * A vector with each element, i.e., `[x, y, z].as_vector()`.
/// * A repeat expression `[expr; N].as_vector()` where `N` is how many times to repeat `expr` in the vector.
#['nargo_doc_primitive vector]
mod primitive_vector {}

/// The string type.
///
/// Denoted `str<N>`, its length `N` must be known at compile-time.
#['nargo_doc_primitive str]
mod primitive_str {}

/// A compile-time type which represents a trait constraint that can be used to search for a trait implementation.
///
/// This is similar syntactically to just the trait itself, but can also contain generic arguments.
/// E.g. [`Eq`](crate::cmp::Eq), [`Default`](crate::default::Default),
/// `BuildHasher<Poseidon2Hasher>`.
///
/// A [`TraitConstraint`] can be obtained using [`Quoted::as_trait_constraint`]:
///
/// ```noir
/// fn main() {
///     comptime {
///         let constraint = quote { Eq }.as_trait_constraint();
///         assert(quote { u32 }.as_type().implements(constraint));
///     }
/// }
/// ```
#['nargo_doc_primitive TraitConstraint]
mod primitive_TraitConstraint {}

/// A compile-time type which represents a trait definition such as `trait Foo { .. }`.
///
/// A [`TraitDefinition`] is passed as the first argument to a macro attribute on top of a trait:
///
/// ```noir
/// #[attr]
/// pub trait Foo {}
///
/// pub struct Bar {}
///
/// impl Foo for Bar {}
///
/// comptime fn attr(trait_definition: TraitDefinition) {
///     let constraint = trait_definition.as_trait_constraint();
///     assert(quote { Bar }.as_type().implements(constraint));
/// }
/// ```
#['nargo_doc_primitive TraitDefinition]
mod primitive_TraitDefinition {}

/// A compile-time type which represents represents a trait implementation such as `impl Foo for Bar { ... }`.
#['nargo_doc_primitive TraitImpl]
mod primitive_TraitImpl {}

/// A compile-time type which represents a type in the source program.
///
/// It can be obtained using [`Quoted::as_type]`:
///
/// ```noir
/// fn main() {
///     comptime {
///         let typ = quote { i32 }.as_type();
///         let integer = typ.as_integer().unwrap();
///         assert(integer.is_signed());
///         assert_eq(integer.bits(), 32);
///     }
/// }
/// ```
#['nargo_doc_primitive Type]
mod primitive_Type {}

/// A compile-time type which represents a type definition such as `struct Foo { ... }`.
///
/// A [`TypeDefinition`] is passed as the first argument to a macro attribute on top of a struct:
///
/// ```noir
/// #[attr]
/// pub struct Foo {}
///
/// comptime fn attr(type_definition: TypeDefinition) {
///     assert_eq(type_definition.name(), quote { Foo });
/// }
/// ```
#['nargo_doc_primitive TypeDefinition]
mod primitive_TypeDefinition {}

/// A compile-time type which represents a resolved and type-checked expression.
///
/// A [`TypedExpr`] can be obtained using [`Expr::resolve`]:
///
/// ```noir
/// fn main() {
///     comptime {
///         let expr = quote { [1, 2, 3].len() }.as_expr().unwrap();
///         let typed = expr.resolve(Option::none());
///         assert_eq(typed.get_type().unwrap(), quote { u32 }.as_type());
///     }
/// }
/// ```
#['nargo_doc_primitive TypedExpr]
mod primitive_TypedExpr {}

/// A compile-time type which represents the syntax of a type.
///
/// That is, it could be for example `[Foo; 3]` without necessarily `Foo` existing in the source program.
#['nargo_doc_primitive UnresolvedType]
mod primitive_UnresolvedType {}
