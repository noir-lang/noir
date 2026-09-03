---
title: Expr
description: Introspect and transform quoted expressions at compile time—inspect structure, resolve types, and modify sub-expressions.
---

`std::meta::expr` contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

## Methods

### as_array

```rust title="as_array" showLineNumbers 
pub comptime fn as_array(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L451-L453" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L451-L453</a></sub></sup>


If this expression is an array, this returns a vector of each element in the array.

### as_assert

```rust title="as_assert" showLineNumbers 
pub comptime fn as_assert(self) -> Option<AssertExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L456-L458" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L456-L458</a></sub></sup>


If this expression is an assert, this returns its predicate and optional message.

### as_assert_eq

```rust title="as_assert_eq" showLineNumbers 
pub comptime fn as_assert_eq(self) -> Option<AssertEqExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L469-L471" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L469-L471</a></sub></sup>


If this expression is an assert_eq, this returns the left-hand-side and right-hand-side
expressions, together with the optional message.

### as_assign

```rust title="as_assign" showLineNumbers 
pub comptime fn as_assign(self) -> Option<AssignExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L481-L483" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L481-L483</a></sub></sup>


If this expression is an assignment, this returns its left- and right-hand sides.

### as_binary_op

```rust title="as_binary_op" showLineNumbers 
pub comptime fn as_binary_op(self) -> Option<BinaryOpExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L494-L496" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L494-L496</a></sub></sup>


If this expression is a binary operator operation `<lhs> <op> <rhs>`,
return the left-hand side, operator, and the right-hand side of the operation.

### as_block

```rust title="as_block" showLineNumbers 
pub comptime fn as_block(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L508-L510" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L508-L510</a></sub></sup>


If this expression is a block `{ stmt1; stmt2; ...; stmtN }`, return
a vector containing each statement.

### as_bool

```rust title="as_bool" showLineNumbers 
pub comptime fn as_bool(self) -> Option<bool> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L514-L516" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L514-L516</a></sub></sup>


If this expression is a boolean literal, return that literal.

### as_cast

```rust title="as_cast" showLineNumbers 
pub comptime fn as_cast(self) -> Option<CastExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L520-L522" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L520-L522</a></sub></sup>


If this expression is a cast expression (`expr as type`), returns the casted
expression and the type to cast to.

### as_comptime

```rust title="as_comptime" showLineNumbers 
pub comptime fn as_comptime(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L534-L536" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L534-L536</a></sub></sup>


If this expression is a `comptime { stmt1; stmt2; ...; stmtN }` block,
return each statement in the block.

### as_constructor

```rust title="as_constructor" showLineNumbers 
pub comptime fn as_constructor(self) -> Option<ConstructorExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L540-L542" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L540-L542</a></sub></sup>


If this expression is a constructor `Type { field1: expr1, ..., fieldN: exprN }`,
return the type and the fields.

### as_for

```rust title="as_for" showLineNumbers 
pub comptime fn as_for(self) -> Option<ForExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L556-L558" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L556-L558</a></sub></sup>


If this expression is a for statement over a single expression, return its identifier,
iterable, and body.

### as_for_range

```rust title="as_for_range" showLineNumbers 
pub comptime fn as_for_range(self) -> Option<ForRangeExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L569-L571" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L569-L571</a></sub></sup>


If this expression is a for statement over a range, return the identifier,
the range start, the range end, whether the range is inclusive, and the
for loop body.

### as_function_call

```rust title="as_function_call" showLineNumbers 
pub comptime fn as_function_call(self) -> Option<FunctionCallExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L588-L590" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L588-L590</a></sub></sup>


If this expression is a function call `foo(arg1, ..., argN)`, return
the function and a vector of each argument.

### as_if

```rust title="as_if" showLineNumbers 
pub comptime fn as_if(self) -> Option<IfExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L602-L604" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L602-L604</a></sub></sup>


If this expression is an `if condition { then_branch } else { else_branch }`,
return the condition, then branch, and else branch. If there is no else branch,
`None` is returned for that branch instead.

### as_index

```rust title="as_index" showLineNumbers 
pub comptime fn as_index(self) -> Option<IndexExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L615-L617" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L615-L617</a></sub></sup>


If this expression is an index into an array `array[index]`, return the
array and the index.

### as_integer

```rust title="as_integer" showLineNumbers 
pub comptime fn as_integer(self) -> Option<IntegerLiteral> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L629-L631" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L629-L631</a></sub></sup>


If this expression is an integer literal, return its value.
Negative integers are encoded as the equivalent negative field value.

### as_lambda

```rust title="as_lambda" showLineNumbers 
pub comptime fn as_lambda(self) -> Option<LambdaExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L639-L641" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L639-L641</a></sub></sup>


If this expression is a lambda, returns the parameters, return type and body.

### as_let

```rust title="as_let" showLineNumbers 
pub comptime fn as_let(self) -> Option<LetExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L659-L661" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L659-L661</a></sub></sup>


If this expression is a let statement, returns the let pattern as an `Expr`,
the optional type annotation, and the assigned expression.

### as_member_access

```rust title="as_member_access" showLineNumbers 
pub comptime fn as_member_access(self) -> Option<MemberAccessExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L672-L674" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L672-L674</a></sub></sup>


If this expression is a member access `foo.bar`, return the struct/tuple
expression and the field. The field will be represented as a quoted value.

### as_method_call

```rust title="as_method_call" showLineNumbers 
pub comptime fn as_method_call(self) -> Option<MethodCallExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L685-L687" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L685-L687</a></sub></sup>


If this expression is a method call `foo.bar::<generic1, ..., genericM>(arg1, ..., argN)`, return
the receiver, method name, a vector of each generic argument, and a vector of each argument.

### as_repeated_element_array

```rust title="as_repeated_element_array" showLineNumbers 
pub comptime fn as_repeated_element_array(self) -> Option<RepeatedElementArrayExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L703-L705" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L703-L705</a></sub></sup>


If this expression is a repeated element array `[elem; length]`, return
the repeated element and the length expressions.

### as_repeated_element_vector

```rust title="as_repeated_element_vector" showLineNumbers 
pub comptime fn as_repeated_element_vector(self) -> Option<RepeatedElementVectorExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L716-L718" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L716-L718</a></sub></sup>


If this expression is a repeated element vector `[elem; length]`, return
the repeated element and the length expressions.

### as_vector

```rust title="as_vector" showLineNumbers 
pub comptime fn as_vector(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L730-L732" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L730-L732</a></sub></sup>


If this expression is a vector literal `@[elem1, ..., elemN]`,
return each element of the vector.

### as_tuple

```rust title="as_tuple" showLineNumbers 
pub comptime fn as_tuple(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L737-L739" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L737-L739</a></sub></sup>


If this expression is a tuple `(field1, ..., fieldN)`,
return each element of the tuple.

### as_unary_op

```rust title="as_unary_op" showLineNumbers 
pub comptime fn as_unary_op(self) -> Option<UnaryOpExpression> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L743-L745" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L743-L745</a></sub></sup>


If this expression is a unary operation `<op> <rhs>`,
return the unary operator as well as the right-hand side expression.

### as_unsafe

```rust title="as_unsafe" showLineNumbers 
pub comptime fn as_unsafe(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L757-L759" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L757-L759</a></sub></sup>


If this expression is an `unsafe { stmt1; ...; stmtN }` block,
return each statement inside in a vector.

### has_semicolon

```rust title="has_semicolon" showLineNumbers 
pub comptime fn has_semicolon(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L778-L780" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L778-L780</a></sub></sup>


`true` if this expression is trailed by a semicolon. E.g.

```
comptime {
    let expr1 = quote { 1 + 2 }.as_expr().unwrap();
    let expr2 = quote { 1 + 2; }.as_expr().unwrap();

    assert(expr1.as_binary_op().is_some());
    assert(expr2.as_binary_op().is_some());

    assert(!expr1.has_semicolon());
    assert(expr2.has_semicolon());
}
```

### is_break

```rust title="is_break" showLineNumbers 
pub comptime fn is_break(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L784-L786" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L784-L786</a></sub></sup>


`true` if this expression is `break`.

### is_continue

```rust title="is_continue" showLineNumbers 
pub comptime fn is_continue(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L790-L792" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L790-L792</a></sub></sup>


`true` if this expression is `continue`.

### quoted

```rust title="quoted" showLineNumbers 
pub comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L795-L797" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L795-L797</a></sub></sup>


Returns this expression as a `Quoted` value. It's the same as `quote { $self }`.

### resolve

```rust title="resolve" showLineNumbers 
pub comptime fn resolve(self, in_function: Option<FunctionDefinition>) -> TypedExpr {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L811-L813" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L811-L813</a></sub></sup>


Resolves and type-checks this expression and returns the result as a `TypedExpr`.

The `in_function` argument specifies where the expression is resolved:
- If it's `none`, the expression is resolved in the function where `resolve` was called
- If it's `some`, the expression is resolved in the given function

If any names used by this expression are not in scope or if there are any type errors,
this will give compiler errors as if the expression was written directly into
the current `comptime` function.
