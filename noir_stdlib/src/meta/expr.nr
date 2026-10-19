//! Contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

use crate::meta::op::BinaryOp;
use crate::meta::op::UnaryOp;
use crate::option::Option;

/// An assert expression decomposed by [`Expr::as_assert`].
///
/// It holds the asserted predicate and the optional message.
///
/// Example:
///
/// ```noir
/// comptime {
///     let assertion = quote { assert(x, "x is false") }.as_expr().unwrap().as_assert().unwrap();
///     assert_eq(assertion.predicate().quoted(), quote { x });
///     assert_eq(assertion.message().unwrap().quoted(), quote { "x is false" });
/// }
/// ```
pub comptime struct AssertExpression {
    predicate: Expr,
    message: Option<Expr>,
}

impl AssertExpression {
    /// The predicate being asserted.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assertion = quote { assert(x) }.as_expr().unwrap().as_assert().unwrap();
    ///     assert_eq(assertion.predicate().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn predicate(self) -> Expr {
        self.predicate
    }

    /// The optional assertion message.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assertion = quote { assert(x) }.as_expr().unwrap().as_assert().unwrap();
    ///     assert(assertion.message().is_none());
    ///
    ///     let assertion = quote { assert(x, "oops") }.as_expr().unwrap().as_assert().unwrap();
    ///     assert_eq(assertion.message().unwrap().quoted(), quote { "oops" });
    /// }
    /// ```
    pub comptime fn message(self) -> Option<Expr> {
        self.message
    }
}

/// An assert-eq expression decomposed by [`Expr::as_assert_eq`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let assertion = quote { assert_eq(x, y, "differ") }.as_expr().unwrap().as_assert_eq().unwrap();
///     assert_eq(assertion.lhs().quoted(), quote { x });
///     assert_eq(assertion.rhs().quoted(), quote { y });
///     assert_eq(assertion.message().unwrap().quoted(), quote { "differ" });
/// }
/// ```
pub comptime struct AssertEqExpression {
    lhs: Expr,
    rhs: Expr,
    message: Option<Expr>,
}

impl AssertEqExpression {
    /// The left-hand side of the equality assertion.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assertion = quote { assert_eq(x, y) }.as_expr().unwrap().as_assert_eq().unwrap();
    ///     assert_eq(assertion.lhs().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn lhs(self) -> Expr {
        self.lhs
    }

    /// The right-hand side of the equality assertion.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assertion = quote { assert_eq(x, y) }.as_expr().unwrap().as_assert_eq().unwrap();
    ///     assert_eq(assertion.rhs().quoted(), quote { y });
    /// }
    /// ```
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }

    /// The optional assertion message.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assertion = quote { assert_eq(x, y) }.as_expr().unwrap().as_assert_eq().unwrap();
    ///     assert(assertion.message().is_none());
    ///
    ///     let assertion = quote { assert_eq(x, y, "differ") }.as_expr().unwrap().as_assert_eq().unwrap();
    ///     assert_eq(assertion.message().unwrap().quoted(), quote { "differ" });
    /// }
    /// ```
    pub comptime fn message(self) -> Option<Expr> {
        self.message
    }
}

/// An assignment expression decomposed by [`Expr::as_assign`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let assignment = quote { x = y }.as_expr().unwrap().as_assign().unwrap();
///     assert_eq(assignment.lhs().quoted(), quote { x });
///     assert_eq(assignment.rhs().quoted(), quote { y });
/// }
/// ```
pub comptime struct AssignExpression {
    lhs: Expr,
    rhs: Expr,
}

impl AssignExpression {
    /// The left-hand side of the assignment.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assignment = quote { x = y }.as_expr().unwrap().as_assign().unwrap();
    ///     assert_eq(assignment.lhs().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn lhs(self) -> Expr {
        self.lhs
    }

    /// The right-hand side of the assignment.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assignment = quote { x = y }.as_expr().unwrap().as_assign().unwrap();
    ///     assert_eq(assignment.rhs().quoted(), quote { y });
    /// }
    /// ```
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }
}

/// A binary operation decomposed by [`Expr::as_binary_op`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let op = quote { x + y }.as_expr().unwrap().as_binary_op().unwrap();
///     assert_eq(op.lhs().quoted(), quote { x });
///     assert(op.operator().is_add());
///     assert_eq(op.rhs().quoted(), quote { y });
/// }
/// ```
pub comptime struct BinaryOpExpression {
    lhs: Expr,
    operator: BinaryOp,
    rhs: Expr,
}

impl BinaryOpExpression {
    /// The left-hand side of the operation.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let op = quote { x + y }.as_expr().unwrap().as_binary_op().unwrap();
    ///     assert_eq(op.lhs().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn lhs(self) -> Expr {
        self.lhs
    }

    /// The binary operator.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let op = quote { x * y }.as_expr().unwrap().as_binary_op().unwrap();
    ///     assert(op.operator().is_multiply());
    /// }
    /// ```
    pub comptime fn operator(self) -> BinaryOp {
        self.operator
    }

    /// The right-hand side of the operation.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let op = quote { x + y }.as_expr().unwrap().as_binary_op().unwrap();
    ///     assert_eq(op.rhs().quoted(), quote { y });
    /// }
    /// ```
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }
}

/// A cast expression decomposed by [`Expr::as_cast`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let cast = quote { x as u8 }.as_expr().unwrap().as_cast().unwrap();
///     assert_eq(cast.expression().quoted(), quote { x });
///     let typ = cast.typ();
///     assert_eq(quote { $typ }, quote { u8 });
/// }
/// ```
pub comptime struct CastExpression {
    expression: Expr,
    typ: UnresolvedType,
}

impl CastExpression {
    /// The expression being cast.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let cast = quote { x as u8 }.as_expr().unwrap().as_cast().unwrap();
    ///     assert_eq(cast.expression().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn expression(self) -> Expr {
        self.expression
    }

    /// The type the expression is cast to.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let typ = quote { x as u8 }.as_expr().unwrap().as_cast().unwrap().typ();
    ///     assert_eq(quote { $typ }, quote { u8 });
    /// }
    /// ```
    pub comptime fn typ(self) -> UnresolvedType {
        self.typ
    }
}

/// A field in a [`ConstructorExpression`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let constructor = quote { Point { x: 1, y: 2 } }.as_expr().unwrap().as_constructor().unwrap();
///     let field = constructor.fields()[1];
///     assert_eq(field.name(), quote { y });
///     assert_eq(field.value().as_integer().unwrap().value(), 2);
/// }
/// ```
pub comptime struct ConstructorField {
    name: Quoted,
    value: Expr,
}

impl ConstructorField {
    /// The field name as a [`Quoted`] identifier.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let constructor = quote { Point { x: 1 } }.as_expr().unwrap().as_constructor().unwrap();
    ///     assert_eq(constructor.fields()[0].name(), quote { x });
    /// }
    /// ```
    pub comptime fn name(self) -> Quoted {
        self.name
    }

    /// The expression assigned to the field.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let constructor = quote { Point { x: 1 } }.as_expr().unwrap().as_constructor().unwrap();
    ///     assert_eq(constructor.fields()[0].value().as_integer().unwrap().value(), 1);
    /// }
    /// ```
    pub comptime fn value(self) -> Expr {
        self.value
    }
}

/// A constructor expression decomposed by [`Expr::as_constructor`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let constructor = quote { Point { x: 1, y: 2 } }.as_expr().unwrap().as_constructor().unwrap();
///     let typ = constructor.typ();
///     assert_eq(quote { $typ }, quote { Point });
///     assert_eq(constructor.fields().len(), 2);
/// }
/// ```
pub comptime struct ConstructorExpression {
    typ: UnresolvedType,
    fields: [ConstructorField],
}

impl ConstructorExpression {
    /// The type being constructed.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let constructor = quote { Point { x: 1 } }.as_expr().unwrap().as_constructor().unwrap();
    ///     let typ = constructor.typ();
    ///     assert_eq(quote { $typ }, quote { Point });
    /// }
    /// ```
    pub comptime fn typ(self) -> UnresolvedType {
        self.typ
    }

    /// The fields in source order.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let constructor = quote { Point { x: 1, y: 2 } }.as_expr().unwrap().as_constructor().unwrap();
    ///     let fields = constructor.fields();
    ///     assert_eq(fields[0].name(), quote { x });
    ///     assert_eq(fields[1].name(), quote { y });
    /// }
    /// ```
    pub comptime fn fields(self) -> [ConstructorField] {
        self.fields
    }
}

/// A for loop over an expression, decomposed by [`Expr::as_for`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let for_loop = quote { for x in array { foo(x) } }.as_expr().unwrap().as_for().unwrap();
///     assert_eq(for_loop.identifier(), quote { x });
///     assert_eq(for_loop.iterable().quoted(), quote { array });
///     assert_eq(for_loop.body().as_block().unwrap()[0].quoted(), quote { foo(x) });
/// }
/// ```
pub comptime struct ForExpression {
    identifier: Quoted,
    iterable: Expr,
    body: Expr,
}

impl ForExpression {
    /// The loop variable as a [`Quoted`] identifier.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for x in array {} }.as_expr().unwrap().as_for().unwrap();
    ///     assert_eq(for_loop.identifier(), quote { x });
    /// }
    /// ```
    pub comptime fn identifier(self) -> Quoted {
        self.identifier
    }

    /// The expression being iterated over.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for x in array {} }.as_expr().unwrap().as_for().unwrap();
    ///     assert_eq(for_loop.iterable().quoted(), quote { array });
    /// }
    /// ```
    pub comptime fn iterable(self) -> Expr {
        self.iterable
    }

    /// The loop body.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for x in array { foo(x) } }.as_expr().unwrap().as_for().unwrap();
    ///     let statements = for_loop.body().as_block().unwrap();
    ///     assert_eq(statements[0].quoted(), quote { foo(x) });
    /// }
    /// ```
    pub comptime fn body(self) -> Expr {
        self.body
    }
}

/// A for loop over a range, decomposed by [`Expr::as_for_range`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let for_loop = quote { for i in 0..n { foo(i) } }.as_expr().unwrap().as_for_range().unwrap();
///     assert_eq(for_loop.identifier(), quote { i });
///     assert_eq(for_loop.start().as_integer().unwrap().value(), 0);
///     assert_eq(for_loop.end().quoted(), quote { n });
///     assert(!for_loop.is_inclusive());
///     assert_eq(for_loop.body().as_block().unwrap()[0].quoted(), quote { foo(i) });
/// }
/// ```
pub comptime struct ForRangeExpression {
    identifier: Quoted,
    start: Expr,
    end: Expr,
    is_inclusive: bool,
    body: Expr,
}

impl ForRangeExpression {
    /// The loop variable as a [`Quoted`] identifier.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for i in 0..n {} }.as_expr().unwrap().as_for_range().unwrap();
    ///     assert_eq(for_loop.identifier(), quote { i });
    /// }
    /// ```
    pub comptime fn identifier(self) -> Quoted {
        self.identifier
    }

    /// The range's lower bound.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for i in 1..n {} }.as_expr().unwrap().as_for_range().unwrap();
    ///     assert_eq(for_loop.start().as_integer().unwrap().value(), 1);
    /// }
    /// ```
    pub comptime fn start(self) -> Expr {
        self.start
    }

    /// The range's upper bound.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for i in 1..n {} }.as_expr().unwrap().as_for_range().unwrap();
    ///     assert_eq(for_loop.end().quoted(), quote { n });
    /// }
    /// ```
    pub comptime fn end(self) -> Expr {
        self.end
    }

    /// Returns `true` if the upper bound is inclusive.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let exclusive = quote { for i in 0..n {} }.as_expr().unwrap().as_for_range().unwrap();
    ///     assert(!exclusive.is_inclusive());
    ///
    ///     let inclusive = quote { for i in 0..=n {} }.as_expr().unwrap().as_for_range().unwrap();
    ///     assert(inclusive.is_inclusive());
    /// }
    /// ```
    pub comptime fn is_inclusive(self) -> bool {
        self.is_inclusive
    }

    /// The loop body.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for i in 0..n { foo(i) } }.as_expr().unwrap().as_for_range().unwrap();
    ///     let statements = for_loop.body().as_block().unwrap();
    ///     assert_eq(statements[0].quoted(), quote { foo(i) });
    /// }
    /// ```
    pub comptime fn body(self) -> Expr {
        self.body
    }
}

/// A function call decomposed by [`Expr::as_function_call`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let call = quote { foo(x, 2) }.as_expr().unwrap().as_function_call().unwrap();
///     assert_eq(call.function().quoted(), quote { foo });
///     assert_eq(call.arguments().len(), 2);
/// }
/// ```
pub comptime struct FunctionCallExpression {
    function: Expr,
    arguments: [Expr],
}

impl FunctionCallExpression {
    /// The expression being called.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo(x) }.as_expr().unwrap().as_function_call().unwrap();
    ///     assert_eq(call.function().quoted(), quote { foo });
    /// }
    /// ```
    pub comptime fn function(self) -> Expr {
        self.function
    }

    /// The call arguments in source order.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo(x, 2) }.as_expr().unwrap().as_function_call().unwrap();
    ///     let arguments = call.arguments();
    ///     assert_eq(arguments[0].quoted(), quote { x });
    ///     assert_eq(arguments[1].as_integer().unwrap().value(), 2);
    /// }
    /// ```
    pub comptime fn arguments(self) -> [Expr] {
        self.arguments
    }
}

/// An if expression decomposed by [`Expr::as_if`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let if_expr = quote { if c { a } else { b } }.as_expr().unwrap().as_if().unwrap();
///     assert_eq(if_expr.condition().quoted(), quote { c });
///     assert_eq(if_expr.consequence().as_block().unwrap()[0].quoted(), quote { a });
///     assert(if_expr.alternative().is_some());
/// }
/// ```
pub comptime struct IfExpression {
    condition: Expr,
    consequence: Expr,
    alternative: Option<Expr>,
}

impl IfExpression {
    /// The condition controlling the expression.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let if_expr = quote { if c { a } }.as_expr().unwrap().as_if().unwrap();
    ///     assert_eq(if_expr.condition().quoted(), quote { c });
    /// }
    /// ```
    pub comptime fn condition(self) -> Expr {
        self.condition
    }

    /// The branch evaluated when the condition is true.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let if_expr = quote { if c { a } }.as_expr().unwrap().as_if().unwrap();
    ///     assert_eq(if_expr.consequence().as_block().unwrap()[0].quoted(), quote { a });
    /// }
    /// ```
    pub comptime fn consequence(self) -> Expr {
        self.consequence
    }

    /// The optional branch evaluated when the condition is false.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let if_expr = quote { if c { a } }.as_expr().unwrap().as_if().unwrap();
    ///     assert(if_expr.alternative().is_none());
    ///
    ///     let if_expr = quote { if c { a } else { b } }.as_expr().unwrap().as_if().unwrap();
    ///     let alternative = if_expr.alternative().unwrap();
    ///     assert_eq(alternative.as_block().unwrap()[0].quoted(), quote { b });
    /// }
    /// ```
    pub comptime fn alternative(self) -> Option<Expr> {
        self.alternative
    }
}

/// An index expression decomposed by [`Expr::as_index`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let index = quote { array[i] }.as_expr().unwrap().as_index().unwrap();
///     assert_eq(index.collection().quoted(), quote { array });
///     assert_eq(index.index().quoted(), quote { i });
/// }
/// ```
pub comptime struct IndexExpression {
    collection: Expr,
    index: Expr,
}

impl IndexExpression {
    /// The collection being indexed.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let index = quote { array[i] }.as_expr().unwrap().as_index().unwrap();
    ///     assert_eq(index.collection().quoted(), quote { array });
    /// }
    /// ```
    pub comptime fn collection(self) -> Expr {
        self.collection
    }

    /// The index expression.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let index = quote { array[i] }.as_expr().unwrap().as_index().unwrap();
    ///     assert_eq(index.index().quoted(), quote { i });
    /// }
    /// ```
    pub comptime fn index(self) -> Expr {
        self.index
    }
}

/// An integer literal decomposed by [`Expr::as_integer`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let literal = quote { 42 }.as_expr().unwrap().as_integer().unwrap();
///     assert_eq(literal.value(), 42);
/// }
/// ```
pub comptime struct IntegerLiteral {
    value: Field,
}

impl IntegerLiteral {
    /// The literal's value as a field.
    ///
    /// Negative integers are encoded as the equivalent negative field value.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     assert_eq(quote { 42 }.as_expr().unwrap().as_integer().unwrap().value(), 42);
    ///     assert_eq(quote { -1i8 }.as_expr().unwrap().as_integer().unwrap().value(), -1);
    /// }
    /// ```
    pub comptime fn value(self) -> Field {
        self.value
    }
}

/// A parameter in a [`LambdaExpression`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let lambda = quote { |x: u32, y| x + y }.as_expr().unwrap().as_lambda().unwrap();
///     let parameters = lambda.parameters();
///     assert_eq(parameters[0].pattern().quoted(), quote { x });
///     let typ = parameters[0].typ().unwrap();
///     assert_eq(quote { $typ }, quote { u32 });
///     assert(parameters[1].typ().is_none());
/// }
/// ```
pub comptime struct LambdaParameter {
    pattern: Expr,
    typ: Option<UnresolvedType>,
}

impl LambdaParameter {
    /// The parameter pattern.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let lambda = quote { |x| x }.as_expr().unwrap().as_lambda().unwrap();
    ///     assert_eq(lambda.parameters()[0].pattern().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn pattern(self) -> Expr {
        self.pattern
    }

    /// The optional parameter type annotation.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let lambda = quote { |x: u32, y| x + y }.as_expr().unwrap().as_lambda().unwrap();
    ///     let typ = lambda.parameters()[0].typ().unwrap();
    ///     assert_eq(quote { $typ }, quote { u32 });
    ///     assert(lambda.parameters()[1].typ().is_none());
    /// }
    /// ```
    pub comptime fn typ(self) -> Option<UnresolvedType> {
        self.typ
    }
}

/// A lambda expression decomposed by [`Expr::as_lambda`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let lambda = quote { |x: u32| -> u32 { foo(x) } }.as_expr().unwrap().as_lambda().unwrap();
///     assert_eq(lambda.parameters().len(), 1);
///     assert(lambda.return_type().is_some());
///     assert_eq(lambda.body().as_block().unwrap()[0].quoted(), quote { foo(x) });
/// }
/// ```
pub comptime struct LambdaExpression {
    parameters: [LambdaParameter],
    return_type: Option<UnresolvedType>,
    body: Expr,
}

impl LambdaExpression {
    /// The lambda parameters in source order.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let lambda = quote { |x, y| x + y }.as_expr().unwrap().as_lambda().unwrap();
    ///     let parameters = lambda.parameters();
    ///     assert_eq(parameters.len(), 2);
    ///     assert_eq(parameters[1].pattern().quoted(), quote { y });
    /// }
    /// ```
    pub comptime fn parameters(self) -> [LambdaParameter] {
        self.parameters
    }

    /// The optional return type annotation.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let lambda = quote { |x| -> u32 { x } }.as_expr().unwrap().as_lambda().unwrap();
    ///     let typ = lambda.return_type().unwrap();
    ///     assert_eq(quote { $typ }, quote { u32 });
    ///
    ///     let lambda = quote { |x| x }.as_expr().unwrap().as_lambda().unwrap();
    ///     assert(lambda.return_type().is_none());
    /// }
    /// ```
    pub comptime fn return_type(self) -> Option<UnresolvedType> {
        self.return_type
    }

    /// The lambda body.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let lambda = quote { |x| foo(x) }.as_expr().unwrap().as_lambda().unwrap();
    ///     assert_eq(lambda.body().quoted(), quote { foo(x) });
    /// }
    /// ```
    pub comptime fn body(self) -> Expr {
        self.body
    }
}

/// A let expression decomposed by [`Expr::as_let`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let let_expr = quote { let x: u32 = foo(); }.as_expr().unwrap().as_let().unwrap();
///     assert_eq(let_expr.pattern().quoted(), quote { x });
///     assert(let_expr.typ().is_some());
///     assert_eq(let_expr.expression().quoted(), quote { foo() });
/// }
/// ```
pub comptime struct LetExpression {
    pattern: Expr,
    typ: Option<UnresolvedType>,
    expression: Expr,
}

impl LetExpression {
    /// The let pattern.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let let_expr = quote { let x = 1; }.as_expr().unwrap().as_let().unwrap();
    ///     assert_eq(let_expr.pattern().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn pattern(self) -> Expr {
        self.pattern
    }

    /// The optional type annotation.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let let_expr = quote { let x: u32 = 1; }.as_expr().unwrap().as_let().unwrap();
    ///     let typ = let_expr.typ().unwrap();
    ///     assert_eq(quote { $typ }, quote { u32 });
    ///
    ///     let let_expr = quote { let x = 1; }.as_expr().unwrap().as_let().unwrap();
    ///     assert(let_expr.typ().is_none());
    /// }
    /// ```
    pub comptime fn typ(self) -> Option<UnresolvedType> {
        self.typ
    }

    /// The expression assigned to the pattern.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let let_expr = quote { let x = foo(); }.as_expr().unwrap().as_let().unwrap();
    ///     assert_eq(let_expr.expression().quoted(), quote { foo() });
    /// }
    /// ```
    pub comptime fn expression(self) -> Expr {
        self.expression
    }
}

/// A member access decomposed by [`Expr::as_member_access`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let access = quote { point.x }.as_expr().unwrap().as_member_access().unwrap();
///     assert_eq(access.object().quoted(), quote { point });
///     assert_eq(access.member(), quote { x });
/// }
/// ```
pub comptime struct MemberAccessExpression {
    object: Expr,
    member: Quoted,
}

impl MemberAccessExpression {
    /// The expression containing the member.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let access = quote { point.x }.as_expr().unwrap().as_member_access().unwrap();
    ///     assert_eq(access.object().quoted(), quote { point });
    /// }
    /// ```
    pub comptime fn object(self) -> Expr {
        self.object
    }

    /// The member name as a [`Quoted`] identifier.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let access = quote { point.x }.as_expr().unwrap().as_member_access().unwrap();
    ///     assert_eq(access.member(), quote { x });
    /// }
    /// ```
    pub comptime fn member(self) -> Quoted {
        self.member
    }
}

/// A method call decomposed by [`Expr::as_method_call`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let call = quote { foo.bar::<u32>(1) }.as_expr().unwrap().as_method_call().unwrap();
///     assert_eq(call.receiver().quoted(), quote { foo });
///     assert_eq(call.method_name(), quote { bar });
///     assert_eq(call.generics().len(), 1);
///     assert_eq(call.arguments().len(), 1);
/// }
/// ```
pub comptime struct MethodCallExpression {
    receiver: Expr,
    method_name: Quoted,
    generics: [UnresolvedType],
    arguments: [Expr],
}

impl MethodCallExpression {
    /// The method receiver.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo.bar() }.as_expr().unwrap().as_method_call().unwrap();
    ///     assert_eq(call.receiver().quoted(), quote { foo });
    /// }
    /// ```
    pub comptime fn receiver(self) -> Expr {
        self.receiver
    }

    /// The method name as a [`Quoted`] identifier.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo.bar() }.as_expr().unwrap().as_method_call().unwrap();
    ///     assert_eq(call.method_name(), quote { bar });
    /// }
    /// ```
    pub comptime fn method_name(self) -> Quoted {
        self.method_name
    }

    /// The explicit generic arguments in source order.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo.bar::<u32>() }.as_expr().unwrap().as_method_call().unwrap();
    ///     let typ = call.generics()[0];
    ///     assert_eq(quote { $typ }, quote { u32 });
    ///
    ///     let call = quote { foo.bar() }.as_expr().unwrap().as_method_call().unwrap();
    ///     assert_eq(call.generics().len(), 0);
    /// }
    /// ```
    pub comptime fn generics(self) -> [UnresolvedType] {
        self.generics
    }

    /// The call arguments in source order.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo.bar(x, 2) }.as_expr().unwrap().as_method_call().unwrap();
    ///     let arguments = call.arguments();
    ///     assert_eq(arguments[0].quoted(), quote { x });
    ///     assert_eq(arguments[1].as_integer().unwrap().value(), 2);
    /// }
    /// ```
    pub comptime fn arguments(self) -> [Expr] {
        self.arguments
    }
}

/// A repeated-element array decomposed by [`Expr::as_repeated_element_array`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let array = quote { [x; 3] }.as_expr().unwrap().as_repeated_element_array().unwrap();
///     assert_eq(array.element().quoted(), quote { x });
///     assert_eq(array.length().as_integer().unwrap().value(), 3);
/// }
/// ```
pub comptime struct RepeatedElementArrayExpression {
    element: Expr,
    length: Expr,
}

impl RepeatedElementArrayExpression {
    /// The repeated element.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let array = quote { [x; 3] }.as_expr().unwrap().as_repeated_element_array().unwrap();
    ///     assert_eq(array.element().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn element(self) -> Expr {
        self.element
    }

    /// The array length expression.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let array = quote { [x; 3] }.as_expr().unwrap().as_repeated_element_array().unwrap();
    ///     assert_eq(array.length().as_integer().unwrap().value(), 3);
    /// }
    /// ```
    pub comptime fn length(self) -> Expr {
        self.length
    }
}

/// A repeated-element vector decomposed by [`Expr::as_repeated_element_vector`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let vector = quote { @[x; 3] }.as_expr().unwrap().as_repeated_element_vector().unwrap();
///     assert_eq(vector.element().quoted(), quote { x });
///     assert_eq(vector.length().as_integer().unwrap().value(), 3);
/// }
/// ```
pub comptime struct RepeatedElementVectorExpression {
    element: Expr,
    length: Expr,
}

impl RepeatedElementVectorExpression {
    /// The repeated element.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let vector = quote { @[x; 3] }.as_expr().unwrap().as_repeated_element_vector().unwrap();
    ///     assert_eq(vector.element().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn element(self) -> Expr {
        self.element
    }

    /// The vector length expression.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let vector = quote { @[x; 3] }.as_expr().unwrap().as_repeated_element_vector().unwrap();
    ///     assert_eq(vector.length().as_integer().unwrap().value(), 3);
    /// }
    /// ```
    pub comptime fn length(self) -> Expr {
        self.length
    }
}

/// A unary operation decomposed by [`Expr::as_unary_op`].
///
/// Example:
///
/// ```noir
/// comptime {
///     let op = quote { -x }.as_expr().unwrap().as_unary_op().unwrap();
///     assert(op.operator().is_minus());
///     assert_eq(op.rhs().quoted(), quote { x });
/// }
/// ```
pub comptime struct UnaryOpExpression {
    operator: UnaryOp,
    rhs: Expr,
}

impl UnaryOpExpression {
    /// The unary operator.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let op = quote { !x }.as_expr().unwrap().as_unary_op().unwrap();
    ///     assert(op.operator().is_not());
    /// }
    /// ```
    pub comptime fn operator(self) -> UnaryOp {
        self.operator
    }

    /// The operand on the right-hand side of the operator.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let op = quote { -x }.as_expr().unwrap().as_unary_op().unwrap();
    ///     assert_eq(op.rhs().quoted(), quote { x });
    /// }
    /// ```
    pub comptime fn rhs(self) -> Expr {
        self.rhs
    }
}

impl Expr {
    /// If this expression is an array literal `[elem1, ..., elemN]`, this returns a vector of each element in the array.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let elements = quote { [1, x] }.as_expr().unwrap().as_array().unwrap();
    ///     assert_eq(elements[0].as_integer().unwrap().value(), 1);
    ///     assert_eq(elements[1].quoted(), quote { x });
    ///
    ///     assert(quote { @[1, x] }.as_expr().unwrap().as_array().is_none());
    /// }
    /// ```
    #[builtin(expr_as_array)]
    // docs:start:as_array
    pub comptime fn as_array(self) -> Option<[Expr]> {}
    // docs:end:as_array

    /// If this expression is an assert, this returns its predicate and optional message.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assertion = quote { assert(x) }.as_expr().unwrap().as_assert().unwrap();
    ///     assert_eq(assertion.predicate().quoted(), quote { x });
    ///
    ///     assert(quote { assert_eq(x, y) }.as_expr().unwrap().as_assert().is_none());
    /// }
    /// ```
    // docs:start:as_assert
    pub comptime fn as_assert(self) -> Option<AssertExpression> {
        // docs:end:as_assert
        self.as_assert_raw().map(|parts: (Expr, Option<Expr>)| {
            AssertExpression { predicate: parts.0, message: parts.1 }
        })
    }

    #[builtin(expr_as_assert)]
    comptime fn as_assert_raw(self) -> Option<(Expr, Option<Expr>)> {}

    /// Decomposes an `assert_eq(lhs, rhs, message)` expression.
    ///
    /// Returns the left- and right-hand sides and the optional message.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assertion = quote { assert_eq(x, y) }.as_expr().unwrap().as_assert_eq().unwrap();
    ///     assert_eq(assertion.lhs().quoted(), quote { x });
    ///     assert_eq(assertion.rhs().quoted(), quote { y });
    ///
    ///     assert(quote { assert(x == y) }.as_expr().unwrap().as_assert_eq().is_none());
    /// }
    /// ```
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
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let assignment = quote { x = 1 }.as_expr().unwrap().as_assign().unwrap();
    ///     assert_eq(assignment.lhs().quoted(), quote { x });
    /// }
    /// ```
    // docs:start:as_assign
    pub comptime fn as_assign(self) -> Option<AssignExpression> {
        // docs:end:as_assign
        self.as_assign_raw().map(|parts: (Expr, Expr)| {
            AssignExpression { lhs: parts.0, rhs: parts.1 }
        })
    }

    #[builtin(expr_as_assign)]
    comptime fn as_assign_raw(self) -> Option<(Expr, Expr)> {}

    /// Decomposes a binary operation `<lhs> <op> <rhs>`.
    ///
    /// Returns the left-hand side, the operator and the right-hand side.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let op = quote { x < y }.as_expr().unwrap().as_binary_op().unwrap();
    ///     assert(op.operator().is_less_than());
    ///
    ///     assert(quote { -x }.as_expr().unwrap().as_binary_op().is_none());
    /// }
    /// ```
    // docs:start:as_binary_op
    pub comptime fn as_binary_op(self) -> Option<BinaryOpExpression> {
        // docs:end:as_binary_op
        self.as_binary_op_raw().map(|parts: (Expr, BinaryOp, Expr)| {
            BinaryOpExpression { lhs: parts.0, operator: parts.1, rhs: parts.2 }
        })
    }

    #[builtin(expr_as_binary_op)]
    comptime fn as_binary_op_raw(self) -> Option<(Expr, BinaryOp, Expr)> {}

    /// Returns each statement of a block `{ stmt1; stmt2; ...; stmtN }`.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let statements = quote { { foo(); bar() } }.as_expr().unwrap().as_block().unwrap();
    ///     assert_eq(statements.len(), 2);
    ///     assert_eq(statements[1].quoted(), quote { bar() });
    /// }
    /// ```
    #[builtin(expr_as_block)]
    // docs:start:as_block
    pub comptime fn as_block(self) -> Option<[Expr]> {}
    // docs:end:as_block

    /// If this expression is a boolean literal, return that literal.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     assert_eq(quote { true }.as_expr().unwrap().as_bool(), Option::some(true));
    ///     assert(quote { 1 }.as_expr().unwrap().as_bool().is_none());
    /// }
    /// ```
    #[builtin(expr_as_bool)]
    // docs:start:as_bool
    pub comptime fn as_bool(self) -> Option<bool> {}
    // docs:end:as_bool

    /// Decomposes a cast `expr as type` into the expression and the type it is cast to.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let cast = quote { x as u8 }.as_expr().unwrap().as_cast().unwrap();
    ///     assert_eq(cast.expression().quoted(), quote { x });
    /// }
    /// ```
    // docs:start:as_cast
    pub comptime fn as_cast(self) -> Option<CastExpression> {
        // docs:end:as_cast
        self.as_cast_raw().map(|parts: (Expr, UnresolvedType)| {
            CastExpression { expression: parts.0, typ: parts.1 }
        })
    }

    #[builtin(expr_as_cast)]
    comptime fn as_cast_raw(self) -> Option<(Expr, UnresolvedType)> {}

    /// Returns each statement of a `comptime { stmt1; stmt2; ...; stmtN }` block.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let statements = quote { comptime { foo(); bar() } }.as_expr().unwrap().as_comptime().unwrap();
    ///     assert_eq(statements.len(), 2);
    ///
    ///     assert(quote { { foo() } }.as_expr().unwrap().as_comptime().is_none());
    /// }
    /// ```
    #[builtin(expr_as_comptime)]
    // docs:start:as_comptime
    pub comptime fn as_comptime(self) -> Option<[Expr]> {}
    // docs:end:as_comptime

    /// Decomposes a constructor `Type { field1: expr1, ..., fieldN: exprN }`.
    ///
    /// Returns the type and the fields.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let constructor = quote { Point { x: 1, y: 2 } }.as_expr().unwrap().as_constructor().unwrap();
    ///     assert_eq(constructor.fields().len(), 2);
    /// }
    /// ```
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

    /// Decomposes a `for` loop over an expression such as an array.
    ///
    /// Returns the loop variable, the iterable and the body. A loop over a range is decomposed by
    /// [`Expr::as_for_range`].
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for x in array {} }.as_expr().unwrap().as_for().unwrap();
    ///     assert_eq(for_loop.iterable().quoted(), quote { array });
    ///
    ///     // A loop over a range is decomposed by `as_for_range` instead.
    ///     assert(quote { for i in 0..3 {} }.as_expr().unwrap().as_for().is_none());
    /// }
    /// ```
    // docs:start:as_for
    pub comptime fn as_for(self) -> Option<ForExpression> {
        // docs:end:as_for
        self.as_for_raw().map(|parts: (Quoted, Expr, Expr)| {
            ForExpression { identifier: parts.0, iterable: parts.1, body: parts.2 }
        })
    }

    #[builtin(expr_as_for)]
    comptime fn as_for_raw(self) -> Option<(Quoted, Expr, Expr)> {}

    /// Decomposes a `for` loop over a range.
    ///
    /// Returns the loop variable, the range start and end, whether the range is inclusive, and the body.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let for_loop = quote { for i in 0..3 {} }.as_expr().unwrap().as_for_range().unwrap();
    ///     assert_eq(for_loop.end().as_integer().unwrap().value(), 3);
    /// }
    /// ```
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

    /// Decomposes a function call `foo(arg1, ..., argN)` into the function and its arguments.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo(42) }.as_expr().unwrap().as_function_call().unwrap();
    ///     assert_eq(call.function().quoted(), quote { foo });
    ///
    ///     // A method call is decomposed by `as_method_call` instead.
    ///     assert(quote { x.foo() }.as_expr().unwrap().as_function_call().is_none());
    /// }
    /// ```
    // docs:start:as_function_call
    pub comptime fn as_function_call(self) -> Option<FunctionCallExpression> {
        // docs:end:as_function_call
        self.as_function_call_raw().map(|parts: (Expr, [Expr])| {
            FunctionCallExpression { function: parts.0, arguments: parts.1 }
        })
    }

    #[builtin(expr_as_function_call)]
    comptime fn as_function_call_raw(self) -> Option<(Expr, [Expr])> {}

    /// Decomposes an `if condition { then_branch } else { else_branch }` expression.
    ///
    /// Returns the condition, the then branch and the else branch, which is `Option::none()` when there
    /// is no `else`.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let if_expr = quote { if c { a } }.as_expr().unwrap().as_if().unwrap();
    ///     assert_eq(if_expr.condition().quoted(), quote { c });
    ///     assert(if_expr.alternative().is_none());
    /// }
    /// ```
    // docs:start:as_if
    pub comptime fn as_if(self) -> Option<IfExpression> {
        // docs:end:as_if
        self.as_if_raw().map(|parts: (Expr, Expr, Option<Expr>)| {
            IfExpression { condition: parts.0, consequence: parts.1, alternative: parts.2 }
        })
    }

    #[builtin(expr_as_if)]
    comptime fn as_if_raw(self) -> Option<(Expr, Expr, Option<Expr>)> {}

    /// Decomposes an index `collection[index]` into the collection and the index.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let index = quote { array[0] }.as_expr().unwrap().as_index().unwrap();
    ///     assert_eq(index.collection().quoted(), quote { array });
    /// }
    /// ```
    // docs:start:as_index
    pub comptime fn as_index(self) -> Option<IndexExpression> {
        // docs:end:as_index
        self.as_index_raw().map(|parts: (Expr, Expr)| {
            IndexExpression { collection: parts.0, index: parts.1 }
        })
    }

    #[builtin(expr_as_index)]
    comptime fn as_index_raw(self) -> Option<(Expr, Expr)> {}

    /// Returns the value of an integer literal.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Negative integers are encoded as the equivalent negative field value, so `-1i8` has a value of
    /// `-1` rather than the two's complement encoding `255`.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     assert_eq(quote { 255 }.as_expr().unwrap().as_integer().unwrap().value(), 255);
    ///     assert_eq(quote { -1i8 }.as_expr().unwrap().as_integer().unwrap().value(), -1);
    ///     assert(quote { x }.as_expr().unwrap().as_integer().is_none());
    /// }
    /// ```
    // docs:start:as_integer
    pub comptime fn as_integer(self) -> Option<IntegerLiteral> {
        // docs:end:as_integer
        self.as_integer_raw().map(|value: Field| IntegerLiteral { value })
    }

    #[builtin(expr_as_integer)]
    comptime fn as_integer_raw(self) -> Option<Field> {}

    /// If this expression is a lambda, returns the parameters, return type and body.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let lambda = quote { |x| x + 1 }.as_expr().unwrap().as_lambda().unwrap();
    ///     assert_eq(lambda.parameters()[0].pattern().quoted(), quote { x });
    /// }
    /// ```
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

    /// Decomposes a `let` statement.
    ///
    /// Returns the pattern as an `Expr`, the optional type annotation and the assigned expression.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let let_expr = quote { let x = 1; }.as_expr().unwrap().as_let().unwrap();
    ///     assert_eq(let_expr.pattern().quoted(), quote { x });
    /// }
    /// ```
    // docs:start:as_let
    pub comptime fn as_let(self) -> Option<LetExpression> {
        // docs:end:as_let
        self.as_let_raw().map(|parts: (Expr, Option<UnresolvedType>, Expr)| {
            LetExpression { pattern: parts.0, typ: parts.1, expression: parts.2 }
        })
    }

    #[builtin(expr_as_let)]
    comptime fn as_let_raw(self) -> Option<(Expr, Option<UnresolvedType>, Expr)> {}

    /// Decomposes a member access `foo.bar`.
    ///
    /// Returns the struct or tuple expression and the member name as a `Quoted` value.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let access = quote { point.x }.as_expr().unwrap().as_member_access().unwrap();
    ///     assert_eq(access.member(), quote { x });
    /// }
    /// ```
    // docs:start:as_member_access
    pub comptime fn as_member_access(self) -> Option<MemberAccessExpression> {
        // docs:end:as_member_access
        self.as_member_access_raw().map(|parts: (Expr, Quoted)| {
            MemberAccessExpression { object: parts.0, member: parts.1 }
        })
    }

    #[builtin(expr_as_member_access)]
    comptime fn as_member_access_raw(self) -> Option<(Expr, Quoted)> {}

    /// Decomposes a method call `foo.bar::<generic1, ..., genericM>(arg1, ..., argN)`.
    ///
    /// Returns the receiver, the method name, the generic arguments and the arguments.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let call = quote { foo.bar(1) }.as_expr().unwrap().as_method_call().unwrap();
    ///     assert_eq(call.method_name(), quote { bar });
    ///
    ///     assert(quote { bar(foo, 1) }.as_expr().unwrap().as_method_call().is_none());
    /// }
    /// ```
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

    /// Decomposes a repeated-element array `[elem; length]` into the element and the length.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let array = quote { [0; 4] }.as_expr().unwrap().as_repeated_element_array().unwrap();
    ///     assert_eq(array.length().as_integer().unwrap().value(), 4);
    ///
    ///     assert(quote { @[0; 4] }.as_expr().unwrap().as_repeated_element_array().is_none());
    /// }
    /// ```
    // docs:start:as_repeated_element_array
    pub comptime fn as_repeated_element_array(self) -> Option<RepeatedElementArrayExpression> {
        // docs:end:as_repeated_element_array
        self.as_repeated_element_array_raw().map(|parts: (Expr, Expr)| {
            RepeatedElementArrayExpression { element: parts.0, length: parts.1 }
        })
    }

    #[builtin(expr_as_repeated_element_array)]
    comptime fn as_repeated_element_array_raw(self) -> Option<(Expr, Expr)> {}

    /// Decomposes a repeated-element vector `@[elem; length]` into the element and the length.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let vector = quote { @[0; 4] }.as_expr().unwrap().as_repeated_element_vector().unwrap();
    ///     assert_eq(vector.length().as_integer().unwrap().value(), 4);
    ///
    ///     assert(quote { [0; 4] }.as_expr().unwrap().as_repeated_element_vector().is_none());
    /// }
    /// ```
    // docs:start:as_repeated_element_vector
    pub comptime fn as_repeated_element_vector(self) -> Option<RepeatedElementVectorExpression> {
        // docs:end:as_repeated_element_vector
        self.as_repeated_element_vector_raw().map(|parts: (Expr, Expr)| {
            RepeatedElementVectorExpression { element: parts.0, length: parts.1 }
        })
    }

    #[builtin(expr_as_repeated_element_vector)]
    comptime fn as_repeated_element_vector_raw(self) -> Option<(Expr, Expr)> {}

    /// Returns each element of a vector literal `@[elem1, ..., elemN]`.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let elements = quote { @[1, 2] }.as_expr().unwrap().as_vector().unwrap();
    ///     assert_eq(elements.len(), 2);
    ///
    ///     assert(quote { [1, 2] }.as_expr().unwrap().as_vector().is_none());
    /// }
    /// ```
    #[builtin(expr_as_vector)]
    // docs:start:as_vector
    pub comptime fn as_vector(self) -> Option<[Expr]> {}
    // docs:end:as_vector

    /// Returns each element of a tuple `(field1, ..., fieldN)`.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let elements = quote { (x, 1) }.as_expr().unwrap().as_tuple().unwrap();
    ///     assert_eq(elements[0].quoted(), quote { x });
    ///     assert_eq(elements[1].as_integer().unwrap().value(), 1);
    /// }
    /// ```
    #[builtin(expr_as_tuple)]
    // docs:start:as_tuple
    pub comptime fn as_tuple(self) -> Option<[Expr]> {}
    // docs:end:as_tuple

    /// Decomposes a unary operation `<op> <rhs>` into the operator and the right-hand side.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// An immutable reference `&x` isn't exposed to comptime code, so it also returns
    /// `Option::none()`.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let op = quote { *x }.as_expr().unwrap().as_unary_op().unwrap();
    ///     assert(op.operator().is_dereference());
    ///
    ///     assert(quote { &x }.as_expr().unwrap().as_unary_op().is_none());
    /// }
    /// ```
    // docs:start:as_unary_op
    pub comptime fn as_unary_op(self) -> Option<UnaryOpExpression> {
        // docs:end:as_unary_op
        self.as_unary_op_raw().map(|parts: (UnaryOp, Expr)| {
            UnaryOpExpression { operator: parts.0, rhs: parts.1 }
        })
    }

    #[builtin(expr_as_unary_op)]
    comptime fn as_unary_op_raw(self) -> Option<(UnaryOp, Expr)> {}

    /// Returns each statement of an `unsafe { stmt1; ...; stmtN }` block.
    /// Returns `Option::none()` if this expression is anything else.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let block = quote { /* Safety: example */ unsafe { foo() } }.as_expr().unwrap();
    ///     let statements = block.as_unsafe().unwrap();
    ///     assert_eq(statements[0].quoted(), quote { foo() });
    /// }
    /// ```
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
    ///     let block = quote { { 1 + 2; 3 } }.as_expr().unwrap();
    ///     let statements = block.as_block().unwrap();
    ///
    ///     assert(statements[0].as_binary_op().is_some());
    ///     assert(statements[0].has_semicolon());
    ///     assert(!statements[1].has_semicolon());
    /// }
    /// ```
    #[builtin(expr_has_semicolon)]
    // docs:start:has_semicolon
    pub comptime fn has_semicolon(self) -> bool {}
    // docs:end:has_semicolon

    /// Returns `true` if this expression is `break`.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let statements = quote { { break; continue; } }.as_expr().unwrap().as_block().unwrap();
    ///     assert(statements[0].is_break());
    ///     assert(!statements[1].is_break());
    /// }
    /// ```
    #[builtin(expr_is_break)]
    // docs:start:is_break
    pub comptime fn is_break(self) -> bool {}
    // docs:end:is_break

    /// Returns `true` if this expression is `continue`.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let statements = quote { { break; continue; } }.as_expr().unwrap().as_block().unwrap();
    ///     assert(!statements[0].is_continue());
    ///     assert(statements[1].is_continue());
    /// }
    /// ```
    #[builtin(expr_is_continue)]
    // docs:start:is_continue
    pub comptime fn is_continue(self) -> bool {}
    // docs:end:is_continue

    /// Returns this expression as a `Quoted` value. It's the same as `quote { $self }`.
    ///
    /// Two `Quoted` values are equal when they print the same. An expression prints with its own
    /// parentheses, so `x + y` re-quoted through this method equals `quote { (x + y) }`, not
    /// `quote { x + y }`.
    ///
    /// Example:
    ///
    /// ```noir
    /// comptime {
    ///     let expr = quote { foo(x) }.as_expr().unwrap();
    ///     assert_eq(expr.quoted(), quote { foo(x) });
    ///
    ///     let sum = quote { x + y }.as_expr().unwrap();
    ///     assert_eq(sum.quoted(), quote { (x + y) });
    /// }
    /// ```
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
    ///         let call = quote { double(3) }.as_expr().unwrap().resolve(Option::none());
    ///         assert_eq(call.get_type().unwrap(), quote { u32 }.as_type());
    ///
    ///         let function = quote { double }.as_expr().unwrap().resolve(Option::none());
    ///         assert_eq(function.as_function_definition().unwrap().name(), quote { double });
    ///     }
    /// }
    /// ```
    #[builtin(expr_resolve)]
    // docs:start:resolve
    pub comptime fn resolve(self, in_function: Option<FunctionDefinition>) -> TypedExpr {}
    // docs:end:resolve
}
