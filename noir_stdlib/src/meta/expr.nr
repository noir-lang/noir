//! Contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

use crate::meta::op::BinaryOp;
use crate::meta::op::UnaryOp;
use crate::option::Option;

impl Expr {
    /// If this expression is an array literal `[elem1, ..., elemN]`, this returns a slice of each element in the array.
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
    /// a slice containing each statement.
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
    /// the range start, the range end and the for loop body.
    #[builtin(expr_as_for_range)]
    // docs:start:as_for_range
    pub comptime fn as_for_range(self) -> Option<(Quoted, Expr, Expr, Expr)> {}
    // docs:end:as_for_range

    /// If this expression is a function call `foo(arg1, ..., argN)`, return
    /// the function and a slice of each argument.
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

    /// If this expression is an integer literal, return the integer as a field
    /// as well as whether the integer is negative (true) or not (false).
    #[builtin(expr_as_integer)]
    // docs:start:as_integer
    pub comptime fn as_integer(self) -> Option<(Field, bool)> {}
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
    /// the receiver, method name, a slice of each generic argument, and a slice of each argument.
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

    /// If this expression is a repeated element slice `[elem; length]`, return
    /// the repeated element and the length expressions.
    #[builtin(expr_as_repeated_element_slice)]
    // docs:start:as_repeated_element_slice
    pub comptime fn as_repeated_element_slice(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_repeated_element_slice

    /// If this expression is a slice literal `&[elem1, ..., elemN]`,
    /// return each element of the slice.
    #[builtin(expr_as_slice)]
    // docs:start:as_slice
    pub comptime fn as_slice(self) -> Option<[Expr]> {}
    // docs:end:as_slice

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
    /// return each statement inside in a slice.
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

    /// Applies a mapping function to this expression and to all of its sub-expressions.
    /// `f` will be applied to each sub-expression first, then applied to the expression itself.
    ///
    /// This happens recursively for every expression within `self`.
    ///
    /// For example, calling `modify` on `(&[1], &[2, 3])` with an `f` that returns `Option::some`
    /// for expressions that are integers, doubling them, would return `(&[2], &[4, 6])`.
    // docs:start:modify
    pub comptime fn modify<Env>(self, f: fn[Env](Expr) -> Option<Expr>) -> Expr {
        // docs:end:modify
        let result = modify_array(self, f);
        let result = result.or_else(|| modify_assert(self, f));
        let result = result.or_else(|| modify_assert_eq(self, f));
        let result = result.or_else(|| modify_assign(self, f));
        let result = result.or_else(|| modify_binary_op(self, f));
        let result = result.or_else(|| modify_block(self, f));
        let result = result.or_else(|| modify_cast(self, f));
        let result = result.or_else(|| modify_comptime(self, f));
        let result = result.or_else(|| modify_constructor(self, f));
        let result = result.or_else(|| modify_if(self, f));
        let result = result.or_else(|| modify_index(self, f));
        let result = result.or_else(|| modify_for(self, f));
        let result = result.or_else(|| modify_for_range(self, f));
        let result = result.or_else(|| modify_lambda(self, f));
        let result = result.or_else(|| modify_let(self, f));
        let result = result.or_else(|| modify_function_call(self, f));
        let result = result.or_else(|| modify_member_access(self, f));
        let result = result.or_else(|| modify_method_call(self, f));
        let result = result.or_else(|| modify_repeated_element_array(self, f));
        let result = result.or_else(|| modify_repeated_element_slice(self, f));
        let result = result.or_else(|| modify_slice(self, f));
        let result = result.or_else(|| modify_tuple(self, f));
        let result = result.or_else(|| modify_unary_op(self, f));
        let result = result.or_else(|| modify_unsafe(self, f));
        if result.is_some() {
            let result = result.unwrap_unchecked();
            let modified = f(result);
            modified.unwrap_or(result)
        } else {
            f(self).unwrap_or(self)
        }
    }

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

comptime fn modify_array<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_array().map(|exprs: [Expr]| {
        let exprs = modify_expressions(exprs, f);
        new_array(exprs)
    })
}

comptime fn modify_assert<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_assert().map(|expr: (Expr, Option<Expr>)| {
        let (predicate, msg) = expr;
        let predicate = predicate.modify(f);
        let msg = msg.map(|msg: Expr| msg.modify(f));
        new_assert(predicate, msg)
    })
}

comptime fn modify_assert_eq<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_assert_eq().map(|expr: (Expr, Expr, Option<Expr>)| {
        let (lhs, rhs, msg) = expr;
        let lhs = lhs.modify(f);
        let rhs = rhs.modify(f);
        let msg = msg.map(|msg: Expr| msg.modify(f));
        new_assert_eq(lhs, rhs, msg)
    })
}

comptime fn modify_assign<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_assign().map(|expr: (Expr, Expr)| {
        let (lhs, rhs) = expr;
        let lhs = lhs.modify(f);
        let rhs = rhs.modify(f);
        new_assign(lhs, rhs)
    })
}

comptime fn modify_binary_op<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_binary_op().map(|expr: (Expr, BinaryOp, Expr)| {
        let (lhs, op, rhs) = expr;
        let lhs = lhs.modify(f);
        let rhs = rhs.modify(f);
        new_binary_op(lhs, op, rhs)
    })
}

comptime fn modify_block<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_block().map(|exprs: [Expr]| {
        let exprs = modify_expressions(exprs, f);
        new_block(exprs)
    })
}

comptime fn modify_cast<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_cast().map(|expr: (Expr, UnresolvedType)| {
        let (expr, typ) = expr;
        let expr = expr.modify(f);
        new_cast(expr, typ)
    })
}

comptime fn modify_comptime<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_comptime().map(|exprs: [Expr]| {
        let exprs = exprs.map(|expr: Expr| expr.modify(f));
        new_comptime(exprs)
    })
}

comptime fn modify_constructor<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_constructor().map(|expr: (UnresolvedType, [(Quoted, Expr)])| {
        let (typ, fields) = expr;
        let fields = fields.map(|field: (Quoted, Expr)| {
            let (name, value) = field;
            (name, value.modify(f))
        });
        new_constructor(typ, fields)
    })
}

comptime fn modify_function_call<Env>(
    expr: Expr,
    f: fn[Env](Expr) -> Option<Expr>,
) -> Option<Expr> {
    expr.as_function_call().map(|expr: (Expr, [Expr])| {
        let (function, arguments) = expr;
        let function = function.modify(f);
        let arguments = arguments.map(|arg: Expr| arg.modify(f));
        new_function_call(function, arguments)
    })
}

comptime fn modify_if<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_if().map(|expr: (Expr, Expr, Option<Expr>)| {
        let (condition, consequence, alternative) = expr;
        let condition = condition.modify(f);
        let consequence = consequence.modify(f);
        let alternative = alternative.map(|alternative: Expr| alternative.modify(f));
        new_if(condition, consequence, alternative)
    })
}

comptime fn modify_index<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_index().map(|expr: (Expr, Expr)| {
        let (object, index) = expr;
        let object = object.modify(f);
        let index = index.modify(f);
        new_index(object, index)
    })
}

comptime fn modify_for<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_for().map(|expr: (Quoted, Expr, Expr)| {
        let (identifier, array, body) = expr;
        let array = array.modify(f);
        let body = body.modify(f);
        new_for(identifier, array, body)
    })
}

comptime fn modify_for_range<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_for_range().map(|expr: (Quoted, Expr, Expr, Expr)| {
        let (identifier, from, to, body) = expr;
        let from = from.modify(f);
        let to = to.modify(f);
        let body = body.modify(f);
        new_for_range(identifier, from, to, body)
    })
}

comptime fn modify_lambda<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_lambda().map(|expr: ([(Expr, Option<UnresolvedType>)], Option<UnresolvedType>, Expr)| {
        let (params, return_type, body) = expr;
        let params =
            params.map(|param: (Expr, Option<UnresolvedType>)| (param.0.modify(f), param.1));
        let body = body.modify(f);
        new_lambda(params, return_type, body)
    })
}

comptime fn modify_let<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_let().map(|expr: (Expr, Option<UnresolvedType>, Expr)| {
        let (pattern, typ, expr) = expr;
        let pattern = pattern.modify(f);
        let expr = expr.modify(f);
        new_let(pattern, typ, expr)
    })
}

comptime fn modify_member_access<Env>(
    expr: Expr,
    f: fn[Env](Expr) -> Option<Expr>,
) -> Option<Expr> {
    expr.as_member_access().map(|expr: (Expr, Quoted)| {
        let (object, name) = expr;
        let object = object.modify(f);
        new_member_access(object, name)
    })
}

comptime fn modify_method_call<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_method_call().map(|expr: (Expr, Quoted, [UnresolvedType], [Expr])| {
        let (object, name, generics, arguments) = expr;
        let object = object.modify(f);
        let arguments = arguments.map(|arg: Expr| arg.modify(f));
        new_method_call(object, name, generics, arguments)
    })
}

comptime fn modify_repeated_element_array<Env>(
    expr: Expr,
    f: fn[Env](Expr) -> Option<Expr>,
) -> Option<Expr> {
    expr.as_repeated_element_array().map(|expr: (Expr, Expr)| {
        let (expr, length) = expr;
        let expr = expr.modify(f);
        let length = length.modify(f);
        new_repeated_element_array(expr, length)
    })
}

comptime fn modify_repeated_element_slice<Env>(
    expr: Expr,
    f: fn[Env](Expr) -> Option<Expr>,
) -> Option<Expr> {
    expr.as_repeated_element_slice().map(|expr: (Expr, Expr)| {
        let (expr, length) = expr;
        let expr = expr.modify(f);
        let length = length.modify(f);
        new_repeated_element_slice(expr, length)
    })
}

comptime fn modify_slice<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_slice().map(|exprs: [Expr]| {
        let exprs = modify_expressions(exprs, f);
        new_slice(exprs)
    })
}

comptime fn modify_tuple<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_tuple().map(|exprs: [Expr]| {
        let exprs = modify_expressions(exprs, f);
        new_tuple(exprs)
    })
}

comptime fn modify_unary_op<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_unary_op().map(|expr: (UnaryOp, Expr)| {
        let (op, rhs) = expr;
        let rhs = rhs.modify(f);
        new_unary_op(op, rhs)
    })
}

comptime fn modify_unsafe<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_unsafe().map(|exprs: [Expr]| {
        let exprs = exprs.map(|expr: Expr| expr.modify(f));
        new_unsafe(exprs)
    })
}

comptime fn modify_expressions<Env>(exprs: [Expr], f: fn[Env](Expr) -> Option<Expr>) -> [Expr] {
    exprs.map(|expr: Expr| expr.modify(f))
}

comptime fn new_array(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { , });
    quote { [$exprs]}.as_expr().unwrap()
}

comptime fn new_assert(predicate: Expr, msg: Option<Expr>) -> Expr {
    if msg.is_some() {
        let msg = msg.unwrap();
        quote { assert($predicate, $msg) }.as_expr().unwrap()
    } else {
        quote { assert($predicate) }.as_expr().unwrap()
    }
}

comptime fn new_assert_eq(lhs: Expr, rhs: Expr, msg: Option<Expr>) -> Expr {
    if msg.is_some() {
        let msg = msg.unwrap();
        quote { assert_eq($lhs, $rhs, $msg) }.as_expr().unwrap()
    } else {
        quote { assert_eq($lhs, $rhs) }.as_expr().unwrap()
    }
}

comptime fn new_assign(lhs: Expr, rhs: Expr) -> Expr {
    quote { $lhs = $rhs }.as_expr().unwrap()
}

comptime fn new_binary_op(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
    let op = op.quoted();
    quote { ($lhs) $op ($rhs) }.as_expr().unwrap()
}

comptime fn new_block(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { ; });
    quote { { $exprs }}.as_expr().unwrap()
}

comptime fn new_cast(expr: Expr, typ: UnresolvedType) -> Expr {
    quote { ($expr) as $typ }.as_expr().unwrap()
}

comptime fn new_comptime(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { ; });
    quote { comptime { $exprs }}.as_expr().unwrap()
}

comptime fn new_constructor(typ: UnresolvedType, fields: [(Quoted, Expr)]) -> Expr {
    let fields = fields
        .map(|field: (Quoted, Expr)| {
            let (name, value) = field;
            quote { $name: $value }
        })
        .join(quote { , });
    quote { $typ { $fields }}.as_expr().unwrap()
}

comptime fn new_if(condition: Expr, consequence: Expr, alternative: Option<Expr>) -> Expr {
    if alternative.is_some() {
        let alternative = alternative.unwrap();
        quote { if $condition { $consequence } else { $alternative }}.as_expr().unwrap()
    } else {
        quote { if $condition { $consequence } }.as_expr().unwrap()
    }
}

comptime fn new_for(identifier: Quoted, array: Expr, body: Expr) -> Expr {
    quote { for $identifier in $array { $body } }.as_expr().unwrap()
}

comptime fn new_for_range(identifier: Quoted, from: Expr, to: Expr, body: Expr) -> Expr {
    quote { for $identifier in $from .. $to { $body } }.as_expr().unwrap()
}

comptime fn new_index(object: Expr, index: Expr) -> Expr {
    quote { $object[$index] }.as_expr().unwrap()
}

comptime fn new_lambda(
    params: [(Expr, Option<UnresolvedType>)],
    return_type: Option<UnresolvedType>,
    body: Expr,
) -> Expr {
    let params = params
        .map(|param: (Expr, Option<UnresolvedType>)| {
            let (name, typ) = param;
            if typ.is_some() {
                let typ = typ.unwrap();
                quote { $name: $typ }
            } else {
                quote { $name }
            }
        })
        .join(quote { , });

    if return_type.is_some() {
        let return_type = return_type.unwrap();
        quote { |$params| -> $return_type { $body } }.as_expr().unwrap()
    } else {
        quote { |$params| { $body } }.as_expr().unwrap()
    }
}

comptime fn new_let(pattern: Expr, typ: Option<UnresolvedType>, expr: Expr) -> Expr {
    if typ.is_some() {
        let typ = typ.unwrap();
        quote { let $pattern : $typ = $expr; }.as_expr().unwrap()
    } else {
        quote { let $pattern = $expr; }.as_expr().unwrap()
    }
}

comptime fn new_member_access(object: Expr, name: Quoted) -> Expr {
    quote { $object.$name }.as_expr().unwrap()
}

comptime fn new_function_call(function: Expr, arguments: [Expr]) -> Expr {
    let arguments = join_expressions(arguments, quote { , });

    quote { $function($arguments) }.as_expr().unwrap()
}

comptime fn new_method_call(
    object: Expr,
    name: Quoted,
    generics: [UnresolvedType],
    arguments: [Expr],
) -> Expr {
    let arguments = join_expressions(arguments, quote { , });

    if generics.len() == 0 {
        quote { $object.$name($arguments) }.as_expr().unwrap()
    } else {
        let generics = generics.map(|generic| quote { $generic }).join(quote { , });
        quote { $object.$name::<$generics>($arguments) }.as_expr().unwrap()
    }
}

comptime fn new_repeated_element_array(expr: Expr, length: Expr) -> Expr {
    quote { [$expr; $length] }.as_expr().unwrap()
}

comptime fn new_repeated_element_slice(expr: Expr, length: Expr) -> Expr {
    quote { &[$expr; $length] }.as_expr().unwrap()
}

comptime fn new_slice(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { , });
    quote { &[$exprs]}.as_expr().unwrap()
}

comptime fn new_tuple(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { , });
    quote { ($exprs) }.as_expr().unwrap()
}

comptime fn new_unary_op(op: UnaryOp, rhs: Expr) -> Expr {
    let op = op.quoted();
    quote { $op($rhs) }.as_expr().unwrap()
}

comptime fn new_unsafe(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { ; });
    quote { unsafe { 
        //@safety: generated by macro
        $exprs 
    }}
        .as_expr()
        .unwrap()
}

comptime fn join_expressions(exprs: [Expr], separator: Quoted) -> Quoted {
    exprs.map(|expr: Expr| expr.quoted()).join(separator)
}
