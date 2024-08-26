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
        let result = result.or_else(|| map_if(self, f));
        let result = result.or_else(|| map_block(self, f));
        // TODO: check the remaining possibilities...
        // though to test this it's easier if we have `as_let`.
        result.unwrap_or(self)
    }
}

fn map_block<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    let exprs = expr.as_block();
    if exprs.is_some() {
        let exprs = exprs.unwrap();
        let exprs = exprs.map(|expr: Expr| expr.map(f));
        Option::some(new_block(exprs))
    } else {
        Option::none()
    }
}

fn map_if<Env>(expr: Expr, f: fn[Env](Expr) -> Option<Expr>) -> Option<Expr> {
    let if_ = expr.as_if();
    if if_.is_some() {
        let (condition, consequence, alternative) = if_.unwrap();
        let condition = condition.map(f);
        let consequence = consequence.map(f);
        let alternative = alternative.and_then(f);
        Option::some(new_if(condition, consequence, alternative))
    } else {
        Option::none()
    }
}

fn new_block(exprs: [Expr]) -> Expr {
    let exprs = exprs.map(|expr| quote { $expr }).join(quote { });
    quote { { $exprs }}.as_expr().unwrap()
}

fn new_if(condition: Expr, consequence: Expr, alternative: Option<Expr>) -> Expr {
    if alternative.is_some() {
        let alternative = alternative.unwrap();
        quote { if $condition { $consequence } else { $alternative }}.as_expr().unwrap()
    } else {
        quote { if $condition { $consequence } }.as_expr().unwrap()
    }
}
