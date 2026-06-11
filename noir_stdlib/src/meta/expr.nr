//! Contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

use crate::meta::op::BinaryOp;
use crate::meta::op::UnaryOp;
use crate::option::Option;

impl Expr {
    /// If this expression is an array literal `[elem1, ..., elemN]`, this returns a vector of each element in the array.
    #[builtin(expr_as_array)]
    // docs:start:as_array
    pub comptime fn as_array(self) -> Option<[Expr]> {}
    // docs:end:as_array

    /// If this expression is an assert, this returns the assert expression and the optional message.
    #[builtin(expr_as_assert)]
    // docs:start:as_assert
    pub comptime fn as_assert(self) -> Option<(Expr, Option<Expr>)> {}
    // docs:end:as_assert

    /// If this expression is an assert_eq, this returns the left-hand-side and right-hand-side
    /// expressions, together with the optional message.
    #[builtin(expr_as_assert_eq)]
    // docs:start:as_assert_eq
    pub comptime fn as_assert_eq(self) -> Option<(Expr, Expr, Option<Expr>)> {}
    // docs:end:as_assert_eq

    /// If this expression is an assignment, this returns a tuple with the left hand side
    /// and right hand side in order.
    #[builtin(expr_as_assign)]
    // docs:start:as_assign
    pub comptime fn as_assign(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_assign

    /// If this expression is a binary operator operation `<lhs> <op> <rhs>`,
    /// return the left-hand side, operator, and the right-hand side of the operation.
    #[builtin(expr_as_binary_op)]
    // docs:start:as_binary_op
    pub comptime fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)> {}
    // docs:end:as_binary_op

    /// If this expression is a block `{ stmt1; stmt2; ...; stmtN }`, return
    /// a vector containing each statement.
    #[builtin(expr_as_block)]
    // docs:start:as_block
    pub comptime fn as_block(self) -> Option<[Expr]> {}
    // docs:end:as_block

    /// If this expression is a boolean literal, return that literal.
    #[builtin(expr_as_bool)]
    // docs:start:as_bool
    pub comptime fn as_bool(self) -> Option<bool> {}
    // docs:end:as_bool

    /// If this expression is a cast expression `expr as type`, returns the casted
    /// expression and the type to cast to.
    // docs:start:as_cast
    #[builtin(expr_as_cast)]
    pub comptime fn as_cast(self) -> Option<(Expr, UnresolvedType)> {}
    // docs:end:as_cast

    /// If this expression is a `comptime { stmt1; stmt2; ...; stmtN }` block,
    /// return each statement in the block.
    #[builtin(expr_as_comptime)]
    // docs:start:as_comptime
    pub comptime fn as_comptime(self) -> Option<[Expr]> {}
    // docs:end:as_comptime

    /// If this expression is a constructor `Type { field1: expr1, ..., fieldN: exprN }`,
    /// return the type and the fields.
    #[builtin(expr_as_constructor)]
    // docs:start:as_constructor
    pub comptime fn as_constructor(self) -> Option<(UnresolvedType, [(Quoted, Expr)])> {}
    // docs:end:as_constructor

    /// If this expression is a for statement over a single expression, return the identifier,
    /// the expression and the for loop body.
    #[builtin(expr_as_for)]
    // docs:start:as_for
    pub comptime fn as_for(self) -> Option<(Quoted, Expr, Expr)> {}
    // docs:end:as_for

    /// If this expression is a for statement over a range, return the identifier,
    /// the range start, the range end, whether the range is inclusive, and the for loop body.
    #[builtin(expr_as_for_range)]
    // docs:start:as_for_range
    pub comptime fn as_for_range(self) -> Option<(Quoted, Expr, Expr, bool, Expr)> {}
    // docs:end:as_for_range

    /// If this expression is a function call `foo(arg1, ..., argN)`, return
    /// the function and a vector of each argument.
    #[builtin(expr_as_function_call)]
    // docs:start:as_function_call
    pub comptime fn as_function_call(self) -> Option<(Expr, [Expr])> {}
    // docs:end:as_function_call

    /// If this expression is an `if condition { then_branch } else { else_branch }`,
    /// return the condition, then branch, and else branch. If there is no else branch,
    /// `None` is returned for that branch instead.
    #[builtin(expr_as_if)]
    // docs:start:as_if
    pub comptime fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)> {}
    // docs:end:as_if

    /// If this expression is an index into an array `array[index]`, return the
    /// array and the index.
    #[builtin(expr_as_index)]
    // docs:start:as_index
    pub comptime fn as_index(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_index

    /// If this expression is an integer literal, return the integer as a field.
    /// Note that negative integers of signed integer types will be encoded
    /// as the equivalent negative field value such that e.g. `-1i8` will
    /// return the `-1` field value rather than the two's complement encoding of `255`.
    #[builtin(expr_as_integer)]
    // docs:start:as_integer
    pub comptime fn as_integer(self) -> Option<Field> {}
    // docs:end:as_integer

    /// If this expression is a lambda, returns the parameters, return type and body.
    #[builtin(expr_as_lambda)]
    // docs:start:as_lambda
    pub comptime fn as_lambda(
        self,
    ) -> Option<([(Expr, Option<UnresolvedType>)], Option<UnresolvedType>, Expr)> {}
    // docs:end:as_lambda

    /// If this expression is a let statement, returns the let pattern as an `Expr`,
    /// the optional type annotation, and the assigned expression.
    #[builtin(expr_as_let)]
    // docs:start:as_let
    pub comptime fn as_let(self) -> Option<(Expr, Option<UnresolvedType>, Expr)> {}
    // docs:end:as_let

    /// If this expression is a member access `foo.bar`, return the struct/tuple
    /// expression and the field. The field will be represented as a quoted value.
    #[builtin(expr_as_member_access)]
    // docs:start:as_member_access
    pub comptime fn as_member_access(self) -> Option<(Expr, Quoted)> {}
    // docs:end:as_member_access

    /// If this expression is a method call `foo.bar::<generic1, ..., genericM>(arg1, ..., argN)`, return
    /// the receiver, method name, a vector of each generic argument, and a vector of each argument.
    #[builtin(expr_as_method_call)]
    // docs:start:as_method_call
    pub comptime fn as_method_call(self) -> Option<(Expr, Quoted, [UnresolvedType], [Expr])> {}
    // docs:end:as_method_call

    /// If this expression is a repeated element array `[elem; length]`, return
    /// the repeated element and the length expressions.
    #[builtin(expr_as_repeated_element_array)]
    // docs:start:as_repeated_element_array
    pub comptime fn as_repeated_element_array(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_repeated_element_array

    /// If this expression is a repeated element vector `[elem; length]`, return
    /// the repeated element and the length expressions.
    #[builtin(expr_as_repeated_element_vector)]
    // docs:start:as_repeated_element_vector
    pub comptime fn as_repeated_element_vector(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_repeated_element_vector

    /// If this expression is a repeated element vector `[elem; length]`, return
    /// the repeated element and the length expressions.
    /// This method is deprecated in favor of `as_repeated_element_vector`
    #[builtin(expr_as_repeated_element_vector)]
    #[deprecated("This method has been renamed to `as_repeated_element_vector`")]
    pub comptime fn as_repeated_element_slice(self) -> Option<(Expr, Expr)> {}

    /// If this expression is a vector literal `@[elem1, ..., elemN]`,
    /// return each element of the vector.
    #[builtin(expr_as_vector)]
    // docs:start:as_vector
    pub comptime fn as_vector(self) -> Option<[Expr]> {}
    // docs:end:as_vector

    /// If this expression is a vector literal `@[elem1, ..., elemN]`,
    /// return each element of the vector.
    /// This method is deprecated in favor of `as_vector`
    #[builtin(expr_as_vector)]
    #[deprecated("This method has been renamed to `as_vector`")]
    pub comptime fn as_slice(self) -> Option<[Expr]> {}

    /// If this expression is a tuple `(field1, ..., fieldN)`,
    /// return each element of the tuple.
    #[builtin(expr_as_tuple)]
    // docs:start:as_tuple
    pub comptime fn as_tuple(self) -> Option<[Expr]> {}
    // docs:end:as_tuple

    /// If this expression is a unary operation `<op> <rhs>`,
    /// return the unary operator as well as the right-hand side expression.
    #[builtin(expr_as_unary_op)]
    // docs:start:as_unary_op
    pub comptime fn as_unary_op(self) -> Option<(UnaryOp, Expr)> {}
    // docs:end:as_unary_op

    /// If this expression is an `unsafe { stmt1; ...; stmtN }` block,
    /// return each statement inside in a vector.
    #[builtin(expr_as_unsafe)]
    // docs:start:as_unsafe
    pub comptime fn as_unsafe(self) -> Option<[Expr]> {}
    // docs:end:as_unsafe

    /// Returns `true` if this expression is trailed by a semicolon.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let expr1 = quote { 1 + 2 }.as_expr().unwrap();
    ///     let expr2 = quote { 1 + 2; }.as_expr().unwrap();
    ///
    ///     assert(expr1.as_binary_op().is_some());
    ///     assert(expr2.as_binary_op().is_some());
    ///
    ///     assert(!expr1.has_semicolon());
    ///     assert(expr2.has_semicolon());
    /// }
    /// ```
    #[builtin(expr_has_semicolon)]
    // docs:start:has_semicolon
    pub comptime fn has_semicolon(self) -> bool {}
    // docs:end:has_semicolon

    /// Returns `true` if this expression is `break`.
    #[builtin(expr_is_break)]
    // docs:start:is_break
    pub comptime fn is_break(self) -> bool {}
    // docs:end:is_break

    /// Returns `true` if this expression is `continue`.
    #[builtin(expr_is_continue)]
    // docs:start:is_continue
    pub comptime fn is_continue(self) -> bool {}
    // docs:end:is_continue

    /// Returns this expression as a `Quoted` value. It's the same as `quote { $self }`.
    // docs:start:quoted
    pub comptime fn quoted(self) -> Quoted {
        // docs:end:quoted
        quote { $self }
    }

    /// Resolves and type-checks this expression and returns the result as a `TypedExpr`.
    ///
    /// The `in_function` argument specifies where the expression is resolved:
    /// - If it's `none`, the expression is resolved in the function where `resolve` was called
    /// - If it's `some`, the expression is resolved in the given function
    ///
    /// If any names used by this expression are not in scope or if there are any type errors,
    /// this will give compiler errors as if the expression was written directly into
    /// the current `comptime` function.
    #[builtin(expr_resolve)]
    // docs:start:resolve
    pub comptime fn resolve(self, in_function: Option<FunctionDefinition>) -> TypedExpr {}
    // docs:end:resolve
}
