use crate::option::Option;
use crate::meta::op::UnaryOp;
use crate::meta::op::BinaryOp;

impl Expr {
    #[builtin(expr_as_array)]
    // docs:start:as_array
    fn as_array(self) -> Option<[Expr]> {}
    // docs:end:as_array

    #[builtin(expr_as_assign)]
    // docs:start:as_assign
    fn as_assign(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_assign

    #[builtin(expr_as_integer)]
    // docs:start:as_integer
    fn as_integer(self) -> Option<(Field, bool)> {}
    // docs:end:as_integer

    #[builtin(expr_as_binary_op)]
    // docs:start:as_binary_op
    fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)> {}
    // docs:end:as_binary_op

    #[builtin(expr_as_block)]
    // docs:start:as_block
    fn as_block(self) -> Option<[Expr]> {}
    // docs:end:as_block

    #[builtin(expr_as_bool)]
    // docs:start:as_bool
    fn as_bool(self) -> Option<bool> {}
    // docs:end:as_bool

    #[builtin(expr_as_cast)]
    fn as_cast(self) -> Option<(Expr, UnresolvedType)> {}

    #[builtin(expr_as_comptime)]
    // docs:start:as_comptime
    fn as_comptime(self) -> Option<[Expr]> {}
    // docs:end:as_comptime

    #[builtin(expr_as_function_call)]
    // docs:start:as_function_call
    fn as_function_call(self) -> Option<(Expr, [Expr])> {}
    // docs:end:as_function_call

    #[builtin(expr_as_if)]
    // docs:start:as_if
    fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)> {}
    // docs:end:as_if

    #[builtin(expr_as_index)]
    // docs:start:as_index
    fn as_index(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_index

    #[builtin(expr_as_member_access)]
    // docs:start:as_member_access
    fn as_member_access(self) -> Option<(Expr, Quoted)> {}
    // docs:end:as_member_access

    #[builtin(expr_as_method_call)]
    // docs:start:as_method_call
    fn as_method_call(self) -> Option<(Expr, Quoted, [UnresolvedType], [Expr])> {}
    // docs:end:as_method_call

    #[builtin(expr_as_repeated_element_array)]
    // docs:start:as_repeated_element_array
    fn as_repeated_element_array(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_repeated_element_array

    #[builtin(expr_as_repeated_element_slice)]
    // docs:start:as_repeated_element_slice
    fn as_repeated_element_slice(self) -> Option<(Expr, Expr)> {}
    // docs:end:as_repeated_element_slice

    #[builtin(expr_as_slice)]
    // docs:start:as_slice
    fn as_slice(self) -> Option<[Expr]> {}
    // docs:end:as_slice

    #[builtin(expr_as_tuple)]
    // docs:start:as_tuple
    fn as_tuple(self) -> Option<[Expr]> {}
    // docs:end:as_tuple

    #[builtin(expr_as_unary_op)]
    // docs:start:as_unary_op
    fn as_unary_op(self) -> Option<(UnaryOp, Expr)> {}
    // docs:end:as_unary_op

    #[builtin(expr_as_unsafe)]
    // docs:start:as_unsafe
    fn as_unsafe(self) -> Option<[Expr]> {}
    // docs:end:as_unsafe

    #[builtin(expr_has_semicolon)]
    // docs:start:has_semicolon
    fn has_semicolon(self) -> bool {}
    // docs:end:has_semicolon

    #[builtin(expr_is_break)]
    // docs:start:is_break
    fn is_break(self) -> bool {}
    // docs:end:is_break

    #[builtin(expr_is_continue)]
    // docs:start:is_continue
    fn is_continue(self) -> bool {}
    // docs:end:is_continue

    fn map<Env>(self, f: fn[Env](Expr) -> Option<Expr>) -> Expr {
        let result = f(self);
        let result = result.or_else(|| map_array(self, f));
        let result = result.or_else(|| map_assign(self, f));
        let result = result.or_else(|| map_binary_op(self, f));
        let result = result.or_else(|| map_block(self, f));
        let result = result.or_else(|| map_cast(self, f));
        let result = result.or_else(|| map_comptime(self, f));
        let result = result.or_else(|| map_if(self, f));
        let result = result.or_else(|| map_index(self, f));
        let result = result.or_else(|| map_function_call(self, f));
        let result = result.or_else(|| map_member_access(self, f));
        let result = result.or_else(|| map_method_call(self, f));
        let result = result.or_else(|| map_repeated_element_array(self, f));
        let result = result.or_else(|| map_repeated_element_slice(self, f));
        let result = result.or_else(|| map_slice(self, f));
        let result = result.or_else(|| map_tuple(self, f));
        let result = result.or_else(|| map_unary_op(self, f));
        let result = result.or_else(|| map_unsafe(self, f));
        result.unwrap_or(self)
    }

    fn quoted(self) -> Quoted {
        quote { $self }
    }
}

fn map_array<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_array().map(
        |exprs: [Expr]| {
        let exprs = map_expressions(exprs, f);
        new_array(exprs)
    }
    )
}

fn map_assign<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_assign().map(
        |expr: (Expr, Expr)| {
        let (lhs, rhs) = expr;
        let lhs = lhs.map(f);
        let rhs = rhs.map(f);
        new_assign(lhs, rhs)
    }
    )
}

fn map_binary_op<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_binary_op().map(
        |expr: (Expr, BinaryOp, Expr)| {
        let (lhs, op, rhs) = expr;
        let lhs = lhs.map(f);
        let rhs = rhs.map(f);
        new_binary_op(lhs, op, rhs)
    }
    )
}

fn map_block<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_block().map(
        |exprs: [Expr]| {
        let exprs = map_expressions(exprs, f);
        new_block(exprs)
    }
    )
}

fn map_cast<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_cast().map(
        |expr: (Expr, UnresolvedType)| {
        let (expr, typ) = expr;
        let expr = expr.map(f);
        new_cast(expr, typ)
    }
    )
}

fn map_comptime<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_comptime().map(
        |exprs: [Expr]| {
        let exprs = exprs.map(|expr: Expr| expr.map(f));
        new_comptime(exprs)
    }
    )
}

fn map_function_call<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_function_call().map(
        |expr: (Expr, [Expr])| {
        let (function, arguments) = expr;
        let function = function.map(f);
        let arguments = arguments.map(|arg: Expr| arg.map(f));
        new_function_call(function, arguments)
    }
    )
}

fn map_if<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_if().map(
        |expr: (Expr, Expr, Option<Expr>)| {
        let (condition, consequence, alternative) = expr;
        let condition = condition.map(f);
        let consequence = consequence.map(f);
        let alternative = alternative.map(|alternative: Expr| alternative.map(f));
        new_if(condition, consequence, alternative)
    }
    )
}

fn map_index<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_index().map(
        |expr: (Expr, Expr)| {
        let (object, index) = expr;
        let object = object.map(f);
        let index = index.map(f);
        new_index(object, index)
    }
    )
}

fn map_member_access<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_member_access().map(
        |expr: (Expr, Quoted)| {
        let (object, name) = expr;
        let object = object.map(f);
        new_member_access(object, name)
    }
    )
}

fn map_method_call<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_method_call().map(
        |expr: (Expr, Quoted, [UnresolvedType], [Expr])| {
        let (object, name, generics, arguments) = expr;
        let object = object.map(f);
        let arguments = arguments.map(|arg: Expr| arg.map(f));
        new_method_call(object, name, generics, arguments)
    }
    )
}

fn map_repeated_element_array<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_repeated_element_array().map(
        |expr: (Expr, Expr)| {
        let (expr, length) = expr;
        let expr = expr.map(f);
        let length = length.map(f);
        new_repeated_element_array(expr, length)
    }
    )
}

fn map_repeated_element_slice<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_repeated_element_slice().map(
        |expr: (Expr, Expr)| {
        let (expr, length) = expr;
        let expr = expr.map(f);
        let length = length.map(f);
        new_repeated_element_slice(expr, length)
    }
    )
}

fn map_slice<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_slice().map(
        |exprs: [Expr]| {
        let exprs = map_expressions(exprs, f);
        new_slice(exprs)
    }
    )
}

fn map_tuple<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_tuple().map(
        |exprs: [Expr]| {
        let exprs = map_expressions(exprs, f);
        new_tuple(exprs)
    }
    )
}

fn map_unary_op<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_unary_op().map(
        |expr: (UnaryOp, Expr)| {
        let (op, rhs) = expr;
        let rhs = rhs.map(f);
        new_unary_op(op, rhs)
    }
    )
}

fn map_unsafe<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    expr.as_unsafe().map(
        |exprs: [Expr]| {
        let exprs = exprs.map(|expr: Expr| expr.map(f));
        new_unsafe(exprs)
    }
    )
}

fn map_expressions<Env>(exprs: [Expr], f: fn[Env](Expr) -> Option<Expr>) -> [Expr] {
    exprs.map(|expr: Expr| expr.map(f))
}

fn new_array(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { , });
    quote { [$exprs]}.as_expr().unwrap()
}

fn new_assign(lhs: Expr, rhs: Expr) -> Expr {
    quote { $lhs = $rhs }.as_expr().unwrap()
}

fn new_binary_op(lhs: Expr, op: BinaryOp, rhs: Expr) -> Expr {
    let op = op.quoted();
    quote { $lhs $op $rhs }.as_expr().unwrap()
}

fn new_block(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { ; });
    quote { { $exprs }}.as_expr().unwrap()
}

fn new_cast(expr: Expr, typ: UnresolvedType) -> Expr {
    quote { $expr as $typ }.as_expr().unwrap()
}

fn new_comptime(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { ; });
    quote { comptime { $exprs }}.as_expr().unwrap()
}

fn new_if(condition: Expr, consequence: Expr, alternative: Option<Expr>) -> Expr {
    if alternative.is_some() {
        let alternative = alternative.unwrap();
        quote { if $condition { $consequence } else { $alternative }}.as_expr().unwrap()
    } else {
        quote { if $condition { $consequence } }.as_expr().unwrap()
    }
}

fn new_index(object: Expr, index: Expr) -> Expr {
    quote { $object[$index] }.as_expr().unwrap()
}

fn new_member_access(object: Expr, name: Quoted) -> Expr {
    quote { $object.$name }.as_expr().unwrap()
}

fn new_function_call(function: Expr, arguments: [Expr]) -> Expr {
    let arguments = join_expressions(arguments, quote { , });

    quote { $function($arguments) }.as_expr().unwrap()
}

fn new_method_call(object: Expr, name: Quoted, generics: [UnresolvedType], arguments: [Expr]) -> Expr {
    let arguments = join_expressions(arguments, quote { , });

    if generics.len() == 0 {
        quote { $object.$name($arguments) }.as_expr().unwrap()
    } else {
        let generics = generics.map(|generic| quote { $generic }).join(quote { , });
        quote { $object.$name::<$generics>($arguments) }.as_expr().unwrap()
    }
}

fn new_repeated_element_array(expr: Expr, length: Expr) -> Expr {
    quote { [$expr; $length] }.as_expr().unwrap()
}

fn new_repeated_element_slice(expr: Expr, length: Expr) -> Expr {
    quote { &[$expr; $length] }.as_expr().unwrap()
}

fn new_slice(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { , });
    quote { &[$exprs]}.as_expr().unwrap()
}

fn new_tuple(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { , });
    quote { ($exprs) }.as_expr().unwrap()
}

fn new_unary_op(op: UnaryOp, rhs: Expr) -> Expr {
    let op = op.quoted();
    quote { $op $rhs }.as_expr().unwrap()
}

fn new_unsafe(exprs: [Expr]) -> Expr {
    let exprs = join_expressions(exprs, quote { ; });
    quote { unsafe { $exprs }}.as_expr().unwrap()
}

fn join_expressions(exprs: [Expr], separator: Quoted) -> Quoted {
    exprs.map(|expr: Expr| expr.quoted()).join(separator)
}
