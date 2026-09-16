//! Contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

use crate::meta::op::BinaryOp;
use crate::meta::op::UnaryOp;
use crate::option::Option;

/// An assert expression decomposed by [`Expr::as_assert`] into its predicate and
/// optional message.
pub comptime struct AssertExpression {
    predicate: Expr,
    message: Option<Expr>,
}

impl AssertExpression {
    /// The predicate being asserted.
    pub comptime fn predicate(self) -> Expr {
        self.predicate
    }

    /// The optional assertion message.
    pub comptime fn message(self) -> Option<Expr> {
        self.message
    }
}

/// An assert-eq expression decomposed by [`Expr::as_assert_eq`].
pub comptime struct AssertEqExpression {
    lhs: Expr,
    rhs: Expr,
    message: Option<Expr>,
}

impl AssertEqExpression {
    /// The left-hand side of the equality assertion.
    pub comptime fn lhs(self) -> Expr {
        self.lhs
    }

    /// The right-hand side of the equality assertion.
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }

    /// The optional assertion message.
    pub comptime fn message(self) -> Option<Expr> {
        self.message
    }
}

/// An assignment expression decomposed by [`Expr::as_assign`].
pub comptime struct AssignExpression {
    lhs: Expr,
    rhs: Expr,
}

impl AssignExpression {
    /// The left-hand side of the assignment.
    pub comptime fn lhs(self) -> Expr {
        self.lhs
    }

    /// The right-hand side of the assignment.
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }
}

/// A binary operation decomposed by [`Expr::as_binary_op`].
pub comptime struct BinaryOpExpression {
    lhs: Expr,
    operator: BinaryOp,
    rhs: Expr,
}

impl BinaryOpExpression {
    /// The left-hand side of the operation.
    pub comptime fn lhs(self) -> Expr {
        self.lhs
    }

    /// The binary operator.
    pub comptime fn operator(self) -> BinaryOp {
        self.operator
    }

    /// The right-hand side of the operation.
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }
}

/// A cast expression decomposed by [`Expr::as_cast`].
pub comptime struct CastExpression {
    expression: Expr,
    typ: UnresolvedType,
}

impl CastExpression {
    /// The expression being cast.
    pub comptime fn expression(self) -> Expr {
        self.expression
    }

    /// The type the expression is cast to.
    pub comptime fn typ(self) -> UnresolvedType {
        self.typ
    }
}

/// A field in a [`ConstructorExpression`].
pub comptime struct ConstructorField {
    name: Quoted,
    value: Expr,
}

impl ConstructorField {
    /// The field name as a [`Quoted`] identifier.
    pub comptime fn name(self) -> Quoted {
        self.name
    }

    /// The expression assigned to the field.
    pub comptime fn value(self) -> Expr {
        self.value
    }
}

/// A constructor expression decomposed by [`Expr::as_constructor`].
pub comptime struct ConstructorExpression {
    typ: UnresolvedType,
    fields: [ConstructorField],
}

impl ConstructorExpression {
    /// The type being constructed.
    pub comptime fn typ(self) -> UnresolvedType {
        self.typ
    }

    /// The fields in source order.
    pub comptime fn fields(self) -> [ConstructorField] {
        self.fields
    }
}

/// A for loop over an expression, decomposed by [`Expr::as_for`].
pub comptime struct ForExpression {
    identifier: Quoted,
    iterable: Expr,
    body: Expr,
}

impl ForExpression {
    /// The loop variable as a [`Quoted`] identifier.
    pub comptime fn identifier(self) -> Quoted {
        self.identifier
    }

    /// The expression being iterated over.
    pub comptime fn iterable(self) -> Expr {
        self.iterable
    }

    /// The loop body.
    pub comptime fn body(self) -> Expr {
        self.body
    }
}

/// A for loop over a range, decomposed by [`Expr::as_for_range`].
pub comptime struct ForRangeExpression {
    identifier: Quoted,
    start: Expr,
    end: Expr,
    is_inclusive: bool,
    body: Expr,
}

impl ForRangeExpression {
    /// The loop variable as a [`Quoted`] identifier.
    pub comptime fn identifier(self) -> Quoted {
        self.identifier
    }

    /// The range's lower bound.
    pub comptime fn start(self) -> Expr {
        self.start
    }

    /// The range's upper bound.
    pub comptime fn end(self) -> Expr {
        self.end
    }

    /// Returns `true` if the upper bound is inclusive.
    pub comptime fn is_inclusive(self) -> bool {
        self.is_inclusive
    }

    /// The loop body.
    pub comptime fn body(self) -> Expr {
        self.body
    }
}

/// A function call decomposed by [`Expr::as_function_call`].
pub comptime struct FunctionCallExpression {
    function: Expr,
    arguments: [Expr],
}

impl FunctionCallExpression {
    /// The expression being called.
    pub comptime fn function(self) -> Expr {
        self.function
    }

    /// The call arguments in source order.
    pub comptime fn arguments(self) -> [Expr] {
        self.arguments
    }
}

/// An if expression decomposed by [`Expr::as_if`].
pub comptime struct IfExpression {
    condition: Expr,
    consequence: Expr,
    alternative: Option<Expr>,
}

impl IfExpression {
    /// The condition controlling the expression.
    pub comptime fn condition(self) -> Expr {
        self.condition
    }

    /// The branch evaluated when the condition is true.
    pub comptime fn consequence(self) -> Expr {
        self.consequence
    }

    /// The optional branch evaluated when the condition is false.
    pub comptime fn alternative(self) -> Option<Expr> {
        self.alternative
    }
}

/// An index expression decomposed by [`Expr::as_index`].
pub comptime struct IndexExpression {
    collection: Expr,
    index: Expr,
}

impl IndexExpression {
    /// The collection being indexed.
    pub comptime fn collection(self) -> Expr {
        self.collection
    }

    /// The index expression.
    pub comptime fn index(self) -> Expr {
        self.index
    }
}

/// An integer literal decomposed by [`Expr::as_integer`].
pub comptime struct IntegerLiteral {
    value: Field,
}

impl IntegerLiteral {
    /// The literal's value as a field.
    ///
    /// Negative integers are encoded as the equivalent negative field value.
    pub comptime fn value(self) -> Field {
        self.value
    }
}

/// A parameter in a [`LambdaExpression`].
pub comptime struct LambdaParameter {
    pattern: Expr,
    typ: Option<UnresolvedType>,
}

impl LambdaParameter {
    /// The parameter pattern.
    pub comptime fn pattern(self) -> Expr {
        self.pattern
    }

    /// The optional parameter type annotation.
    pub comptime fn typ(self) -> Option<UnresolvedType> {
        self.typ
    }
}

/// A lambda expression decomposed by [`Expr::as_lambda`].
pub comptime struct LambdaExpression {
    parameters: [LambdaParameter],
    return_type: Option<UnresolvedType>,
    body: Expr,
}

impl LambdaExpression {
    /// The lambda parameters in source order.
    pub comptime fn parameters(self) -> [LambdaParameter] {
        self.parameters
    }

    /// The optional return type annotation.
    pub comptime fn return_type(self) -> Option<UnresolvedType> {
        self.return_type
    }

    /// The lambda body.
    pub comptime fn body(self) -> Expr {
        self.body
    }
}

/// A let expression decomposed by [`Expr::as_let`].
pub comptime struct LetExpression {
    pattern: Expr,
    typ: Option<UnresolvedType>,
    expression: Expr,
}

impl LetExpression {
    /// The let pattern.
    pub comptime fn pattern(self) -> Expr {
        self.pattern
    }

    /// The optional type annotation.
    pub comptime fn typ(self) -> Option<UnresolvedType> {
        self.typ
    }

    /// The expression assigned to the pattern.
    pub comptime fn expression(self) -> Expr {
        self.expression
    }
}

/// A member access decomposed by [`Expr::as_member_access`].
pub comptime struct MemberAccessExpression {
    object: Expr,
    member: Quoted,
}

impl MemberAccessExpression {
    /// The expression containing the member.
    pub comptime fn object(self) -> Expr {
        self.object
    }

    /// The member name as a [`Quoted`] identifier.
    pub comptime fn member(self) -> Quoted {
        self.member
    }
}

/// A method call decomposed by [`Expr::as_method_call`].
pub comptime struct MethodCallExpression {
    receiver: Expr,
    method_name: Quoted,
    generics: [UnresolvedType],
    arguments: [Expr],
}

impl MethodCallExpression {
    /// The method receiver.
    pub comptime fn receiver(self) -> Expr {
        self.receiver
    }

    /// The method name as a [`Quoted`] identifier.
    pub comptime fn method_name(self) -> Quoted {
        self.method_name
    }

    /// The explicit generic arguments in source order.
    pub comptime fn generics(self) -> [UnresolvedType] {
        self.generics
    }

    /// The call arguments in source order.
    pub comptime fn arguments(self) -> [Expr] {
        self.arguments
    }
}

/// A repeated-element array decomposed by [`Expr::as_repeated_element_array`].
pub comptime struct RepeatedElementArrayExpression {
    element: Expr,
    length: Expr,
}

impl RepeatedElementArrayExpression {
    /// The repeated element.
    pub comptime fn element(self) -> Expr {
        self.element
    }

    /// The array length expression.
    pub comptime fn length(self) -> Expr {
        self.length
    }
}

/// A repeated-element vector decomposed by [`Expr::as_repeated_element_vector`].
pub comptime struct RepeatedElementVectorExpression {
    element: Expr,
    length: Expr,
}

impl RepeatedElementVectorExpression {
    /// The repeated element.
    pub comptime fn element(self) -> Expr {
        self.element
    }

    /// The vector length expression.
    pub comptime fn length(self) -> Expr {
        self.length
    }
}

/// A unary operation decomposed by [`Expr::as_unary_op`].
pub comptime struct UnaryOpExpression {
    operator: UnaryOp,
    rhs: Expr,
}

impl UnaryOpExpression {
    /// The unary operator.
    pub comptime fn operator(self) -> UnaryOp {
        self.operator
    }

    /// The operand on the right-hand side of the operator.
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }
}

impl Expr {
    /// If this expression is an array literal `[elem1, ..., elemN]`, this returns a vector of each element in the array.
    #[builtin(expr_as_array)]
    // docs:start:as_array
    pub comptime fn as_array(self) -> Option<[Expr]> {}
    // docs:end:as_array

    /// If this expression is an assert, this returns its predicate and optional message.
    // docs:start:as_assert
    pub comptime fn as_assert(self) -> Option<AssertExpression> {
        // docs:end:as_assert
        self.as_assert_raw().map(|parts: (Expr, Option<Expr>)| {
            AssertExpression { predicate: parts.0, message: parts.1 }
        })
    }

    #[builtin(expr_as_assert)]
    comptime fn as_assert_raw(self) -> Option<(Expr, Option<Expr>)> {}

    /// If this expression is an assert_eq, this returns the left-hand-side and right-hand-side
    /// expressions, together with the optional message.
    // docs:start:as_assert_eq
    pub comptime fn as_assert_eq(self) -> Option<AssertEqExpression> {
        // docs:end:as_assert_eq
        self.as_assert_eq_raw().map(|parts: (Expr, Expr, Option<Expr>)| {
            AssertEqExpression { lhs: parts.0, rhs: parts.1, message: parts.2 }
        })
    }

    #[builtin(expr_as_assert_eq)]
    comptime fn as_assert_eq_raw(self) -> Option<(Expr, Expr, Option<Expr>)> {}

    /// If this expression is an assignment, this returns its left- and right-hand sides.
    // docs:start:as_assign
    pub comptime fn as_assign(self) -> Option<AssignExpression> {
        // docs:end:as_assign
        self.as_assign_raw().map(|parts: (Expr, Expr)| {
            AssignExpression { lhs: parts.0, rhs: parts.1 }
        })
    }

    #[builtin(expr_as_assign)]
    comptime fn as_assign_raw(self) -> Option<(Expr, Expr)> {}

    /// If this expression is a binary operator operation `<lhs> <op> <rhs>`,
    /// return the left-hand side, operator, and the right-hand side of the operation.
    // docs:start:as_binary_op
    pub comptime fn as_binary_op(self) -> Option<BinaryOpExpression> {
        // docs:end:as_binary_op
        self.as_binary_op_raw().map(|parts: (Expr, BinaryOp, Expr)| {
            BinaryOpExpression { lhs: parts.0, operator: parts.1, rhs: parts.2 }
        })
    }

    #[builtin(expr_as_binary_op)]
    comptime fn as_binary_op_raw(self) -> Option<(Expr, BinaryOp, Expr)> {}

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
    pub comptime fn as_cast(self) -> Option<CastExpression> {
        // docs:end:as_cast
        self.as_cast_raw().map(|parts: (Expr, UnresolvedType)| {
            CastExpression { expression: parts.0, typ: parts.1 }
        })
    }

    #[builtin(expr_as_cast)]
    comptime fn as_cast_raw(self) -> Option<(Expr, UnresolvedType)> {}

    /// If this expression is a `comptime { stmt1; stmt2; ...; stmtN }` block,
    /// return each statement in the block.
    #[builtin(expr_as_comptime)]
    // docs:start:as_comptime
    pub comptime fn as_comptime(self) -> Option<[Expr]> {}
    // docs:end:as_comptime

    /// If this expression is a constructor `Type { field1: expr1, ..., fieldN: exprN }`,
    /// return the type and the fields.
    // docs:start:as_constructor
    pub comptime fn as_constructor(self) -> Option<ConstructorExpression> {
        // docs:end:as_constructor
        self.as_constructor_raw().map(|parts: (UnresolvedType, [(Quoted, Expr)])| {
            let fields = parts.1.map(|field: (Quoted, Expr)| {
                ConstructorField { name: field.0, value: field.1 }
            });
            ConstructorExpression { typ: parts.0, fields }
        })
    }

    #[builtin(expr_as_constructor)]
    comptime fn as_constructor_raw(self) -> Option<(UnresolvedType, [(Quoted, Expr)])> {}

    /// If this expression is a for statement over a single expression, return its identifier,
    /// iterable, and body.
    // docs:start:as_for
    pub comptime fn as_for(self) -> Option<ForExpression> {
        // docs:end:as_for
        self.as_for_raw().map(|parts: (Quoted, Expr, Expr)| {
            ForExpression { identifier: parts.0, iterable: parts.1, body: parts.2 }
        })
    }

    #[builtin(expr_as_for)]
    comptime fn as_for_raw(self) -> Option<(Quoted, Expr, Expr)> {}

    /// If this expression is a for statement over a range, return the identifier,
    /// the range start, the range end, whether the range is inclusive, and the for loop body.
    // docs:start:as_for_range
    pub comptime fn as_for_range(self) -> Option<ForRangeExpression> {
        // docs:end:as_for_range
        self.as_for_range_raw().map(|parts: (Quoted, Expr, Expr, bool, Expr)| {
            ForRangeExpression {
                identifier: parts.0,
                start: parts.1,
                end: parts.2,
                is_inclusive: parts.3,
                body: parts.4,
            }
        })
    }

    #[builtin(expr_as_for_range)]
    comptime fn as_for_range_raw(self) -> Option<(Quoted, Expr, Expr, bool, Expr)> {}

    /// If this expression is a function call `foo(arg1, ..., argN)`, return
    /// the function and a vector of each argument.
    // docs:start:as_function_call
    pub comptime fn as_function_call(self) -> Option<FunctionCallExpression> {
        // docs:end:as_function_call
        self.as_function_call_raw().map(|parts: (Expr, [Expr])| {
            FunctionCallExpression { function: parts.0, arguments: parts.1 }
        })
    }

    #[builtin(expr_as_function_call)]
    comptime fn as_function_call_raw(self) -> Option<(Expr, [Expr])> {}

    /// If this expression is an `if condition { then_branch } else { else_branch }`,
    /// return the condition, then branch, and else branch. If there is no else branch,
    /// `None` is returned for that branch instead.
    // docs:start:as_if
    pub comptime fn as_if(self) -> Option<IfExpression> {
        // docs:end:as_if
        self.as_if_raw().map(|parts: (Expr, Expr, Option<Expr>)| {
            IfExpression { condition: parts.0, consequence: parts.1, alternative: parts.2 }
        })
    }

    #[builtin(expr_as_if)]
    comptime fn as_if_raw(self) -> Option<(Expr, Expr, Option<Expr>)> {}

    /// If this expression is an index into an array `array[index]`, return the
    /// array and the index.
    // docs:start:as_index
    pub comptime fn as_index(self) -> Option<IndexExpression> {
        // docs:end:as_index
        self.as_index_raw().map(|parts: (Expr, Expr)| {
            IndexExpression { collection: parts.0, index: parts.1 }
        })
    }

    #[builtin(expr_as_index)]
    comptime fn as_index_raw(self) -> Option<(Expr, Expr)> {}

    /// If this expression is an integer literal, return its value. Negative integers are encoded
    /// as the equivalent negative field value such that e.g. `-1i8` has a value of `-1` rather
    /// than the two's complement encoding of `255`.
    // docs:start:as_integer
    pub comptime fn as_integer(self) -> Option<IntegerLiteral> {
        // docs:end:as_integer
        self.as_integer_raw().map(|value: Field| IntegerLiteral { value })
    }

    #[builtin(expr_as_integer)]
    comptime fn as_integer_raw(self) -> Option<Field> {}

    /// If this expression is a lambda, returns the parameters, return type and body.
    // docs:start:as_lambda
    pub comptime fn as_lambda(self) -> Option<LambdaExpression> {
        // docs:end:as_lambda
        self
            .as_lambda_raw()
            .map(|parts: ([(Expr, Option<UnresolvedType>)], Option<UnresolvedType>, Expr)| {
                let parameters = parts.0.map(|parameter: (Expr, Option<UnresolvedType>)| {
                    LambdaParameter { pattern: parameter.0, typ: parameter.1 }
                });
                LambdaExpression { parameters, return_type: parts.1, body: parts.2 }
            })
    }

    #[builtin(expr_as_lambda)]
    comptime fn as_lambda_raw(
        self,
    ) -> Option<([(Expr, Option<UnresolvedType>)], Option<UnresolvedType>, Expr)> {}

    /// If this expression is a let statement, returns the let pattern as an `Expr`,
    /// the optional type annotation, and the assigned expression.
    // docs:start:as_let
    pub comptime fn as_let(self) -> Option<LetExpression> {
        // docs:end:as_let
        self.as_let_raw().map(|parts: (Expr, Option<UnresolvedType>, Expr)| {
            LetExpression { pattern: parts.0, typ: parts.1, expression: parts.2 }
        })
    }

    #[builtin(expr_as_let)]
    comptime fn as_let_raw(self) -> Option<(Expr, Option<UnresolvedType>, Expr)> {}

    /// If this expression is a member access `foo.bar`, return the struct/tuple
    /// expression and the field. The field will be represented as a quoted value.
    // docs:start:as_member_access
    pub comptime fn as_member_access(self) -> Option<MemberAccessExpression> {
        // docs:end:as_member_access
        self.as_member_access_raw().map(|parts: (Expr, Quoted)| {
            MemberAccessExpression { object: parts.0, member: parts.1 }
        })
    }

    #[builtin(expr_as_member_access)]
    comptime fn as_member_access_raw(self) -> Option<(Expr, Quoted)> {}

    /// If this expression is a method call `foo.bar::<generic1, ..., genericM>(arg1, ..., argN)`, return
    /// the receiver, method name, a vector of each generic argument, and a vector of each argument.
    // docs:start:as_method_call
    pub comptime fn as_method_call(self) -> Option<MethodCallExpression> {
        // docs:end:as_method_call
        self.as_method_call_raw().map(|parts: (Expr, Quoted, [UnresolvedType], [Expr])| {
            MethodCallExpression {
                receiver: parts.0,
                method_name: parts.1,
                generics: parts.2,
                arguments: parts.3,
            }
        })
    }

    #[builtin(expr_as_method_call)]
    comptime fn as_method_call_raw(self) -> Option<(Expr, Quoted, [UnresolvedType], [Expr])> {}

    /// If this expression is a repeated element array `[elem; length]`, return
    /// the repeated element and the length expressions.
    // docs:start:as_repeated_element_array
    pub comptime fn as_repeated_element_array(self) -> Option<RepeatedElementArrayExpression> {
        // docs:end:as_repeated_element_array
        self.as_repeated_element_array_raw().map(|parts: (Expr, Expr)| {
            RepeatedElementArrayExpression { element: parts.0, length: parts.1 }
        })
    }

    #[builtin(expr_as_repeated_element_array)]
    comptime fn as_repeated_element_array_raw(self) -> Option<(Expr, Expr)> {}

    /// If this expression is a repeated element vector `[elem; length]`, return
    /// the repeated element and the length expressions.
    // docs:start:as_repeated_element_vector
    pub comptime fn as_repeated_element_vector(self) -> Option<RepeatedElementVectorExpression> {
        // docs:end:as_repeated_element_vector
        self.as_repeated_element_vector_raw().map(|parts: (Expr, Expr)| {
            RepeatedElementVectorExpression { element: parts.0, length: parts.1 }
        })
    }

    #[builtin(expr_as_repeated_element_vector)]
    comptime fn as_repeated_element_vector_raw(self) -> Option<(Expr, Expr)> {}

    /// If this expression is a vector literal `@[elem1, ..., elemN]`,
    /// return each element of the vector.
    #[builtin(expr_as_vector)]
    // docs:start:as_vector
    pub comptime fn as_vector(self) -> Option<[Expr]> {}
    // docs:end:as_vector

    /// If this expression is a tuple `(field1, ..., fieldN)`,
    /// return each element of the tuple.
    #[builtin(expr_as_tuple)]
    // docs:start:as_tuple
    pub comptime fn as_tuple(self) -> Option<[Expr]> {}
    // docs:end:as_tuple

    /// If this expression is a unary operation `<op> <rhs>`,
    /// return the unary operator as well as the right-hand side expression.
    // docs:start:as_unary_op
    pub comptime fn as_unary_op(self) -> Option<UnaryOpExpression> {
        // docs:end:as_unary_op
        self.as_unary_op_raw().map(|parts: (UnaryOp, Expr)| {
            UnaryOpExpression { operator: parts.0, rhs: parts.1 }
        })
    }

    #[builtin(expr_as_unary_op)]
    comptime fn as_unary_op_raw(self) -> Option<(UnaryOp, Expr)> {}

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
