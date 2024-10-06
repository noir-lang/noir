---
title: Expr
---

`std::meta::expr` contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

## Methods

### as_array

```rust title="as_array" showLineNumbers 
comptime fn as_array(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L10-L12" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L10-L12</a></sub></sup>


If this expression is an array, this returns a slice of each element in the array.

### as_assert

```rust title="as_assert" showLineNumbers 
comptime fn as_assert(self) -> Option<(Expr, Option<Expr>)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L16-L18" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L16-L18</a></sub></sup>


If this expression is an assert, this returns the assert expression and the optional message.

### as_assert_eq

```rust title="as_assert_eq" showLineNumbers 
comptime fn as_assert_eq(self) -> Option<(Expr, Expr, Option<Expr>)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L23-L25" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L23-L25</a></sub></sup>


If this expression is an assert_eq, this returns the left-hand-side and right-hand-side
expressions, together with the optional message.

### as_assign

```rust title="as_assign" showLineNumbers 
comptime fn as_assign(self) -> Option<(Expr, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L30-L32" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L30-L32</a></sub></sup>


If this expression is an assignment, this returns a tuple with the left hand side
and right hand side in order.

### as_binary_op

```rust title="as_binary_op" showLineNumbers 
comptime fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L37-L39" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L37-L39</a></sub></sup>


If this expression is a binary operator operation `<lhs> <op> <rhs>`,
return the left-hand side, operator, and the right-hand side of the operation.

### as_block

```rust title="as_block" showLineNumbers 
comptime fn as_block(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L44-L46" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L44-L46</a></sub></sup>


If this expression is a block `{ stmt1; stmt2; ...; stmtN }`, return
a slice containing each statement.

### as_bool

```rust title="as_bool" showLineNumbers 
comptime fn as_bool(self) -> Option<bool> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L50-L52" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L50-L52</a></sub></sup>


If this expression is a boolean literal, return that literal.

### as_cast

```rust title="as_cast" showLineNumbers 
#[builtin(expr_as_cast)]
    comptime fn as_cast(self) -> Option<(Expr, UnresolvedType)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L56-L59" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L56-L59</a></sub></sup>


If this expression is a cast expression (`expr as type`), returns the casted
expression and the type to cast to.

### as_comptime

```rust title="as_comptime" showLineNumbers 
comptime fn as_comptime(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L64-L66" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L64-L66</a></sub></sup>


If this expression is a `comptime { stmt1; stmt2; ...; stmtN }` block,
return each statement in the block.

### as_constructor

```rust title="as_constructor" showLineNumbers 
comptime fn as_constructor(self) -> Option<(UnresolvedType, [(Quoted, Expr)])> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L71-L73" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L71-L73</a></sub></sup>


If this expression is a constructor `Type { field1: expr1, ..., fieldN: exprN }`,
return the type and the fields.

### as_for

```rust title="as_for" showLineNumbers 
comptime fn as_for(self) -> Option<(Quoted, Expr, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L78-L80" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L78-L80</a></sub></sup>


If this expression is a for statement over a single expression, return the identifier,
the expression and the for loop body.

### as_for_range

```rust title="as_for" showLineNumbers 
comptime fn as_for(self) -> Option<(Quoted, Expr, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L78-L80" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L78-L80</a></sub></sup>


If this expression is a for statement over a range, return the identifier,
the range start, the range end and the for loop body.

### as_function_call

```rust title="as_function_call" showLineNumbers 
comptime fn as_function_call(self) -> Option<(Expr, [Expr])> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L92-L94" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L92-L94</a></sub></sup>


If this expression is a function call `foo(arg1, ..., argN)`, return
the function and a slice of each argument.

### as_if

```rust title="as_if" showLineNumbers 
comptime fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L100-L102" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L100-L102</a></sub></sup>


If this expression is an `if condition { then_branch } else { else_branch }`,
return the condition, then branch, and else branch. If there is no else branch,
`None` is returned for that branch instead.

### as_index

```rust title="as_index" showLineNumbers 
comptime fn as_index(self) -> Option<(Expr, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L107-L109" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L107-L109</a></sub></sup>


If this expression is an index into an array `array[index]`, return the
array and the index.

### as_integer

```rust title="as_integer" showLineNumbers 
comptime fn as_integer(self) -> Option<(Field, bool)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L114-L116" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L114-L116</a></sub></sup>


If this expression is an integer literal, return the integer as a field
as well as whether the integer is negative (true) or not (false).

### as_lambda

```rust title="as_lambda" showLineNumbers 
comptime fn as_lambda(self) -> Option<([(Expr, Option<UnresolvedType>)], Option<UnresolvedType>, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L120-L122" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L120-L122</a></sub></sup>


If this expression is a lambda, returns the parameters, return type and body.

### as_let

```rust title="as_let" showLineNumbers 
comptime fn as_let(self) -> Option<(Expr, Option<UnresolvedType>, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L127-L129" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L127-L129</a></sub></sup>


If this expression is a let statement, returns the let pattern as an `Expr`,
the optional type annotation, and the assigned expression.

### as_member_access

```rust title="as_member_access" showLineNumbers 
comptime fn as_member_access(self) -> Option<(Expr, Quoted)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L134-L136" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L134-L136</a></sub></sup>


If this expression is a member access `foo.bar`, return the struct/tuple
expression and the field. The field will be represented as a quoted value.

### as_method_call

```rust title="as_method_call" showLineNumbers 
comptime fn as_method_call(self) -> Option<(Expr, Quoted, [UnresolvedType], [Expr])> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L141-L143" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L141-L143</a></sub></sup>


If this expression is a method call `foo.bar::<generic1, ..., genericM>(arg1, ..., argN)`, return
the receiver, method name, a slice of each generic argument, and a slice of each argument.

### as_repeated_element_array

```rust title="as_repeated_element_array" showLineNumbers 
comptime fn as_repeated_element_array(self) -> Option<(Expr, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L148-L150" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L148-L150</a></sub></sup>


If this expression is a repeated element array `[elem; length]`, return
the repeated element and the length expressions.

### as_repeated_element_slice

```rust title="as_repeated_element_slice" showLineNumbers 
comptime fn as_repeated_element_slice(self) -> Option<(Expr, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L155-L157" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L155-L157</a></sub></sup>


If this expression is a repeated element slice `[elem; length]`, return
the repeated element and the length expressions.

### as_slice

```rust title="as_slice" showLineNumbers 
comptime fn as_slice(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L162-L164" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L162-L164</a></sub></sup>


If this expression is a slice literal `&[elem1, ..., elemN]`,
return each element of the slice.

### as_tuple

```rust title="as_tuple" showLineNumbers 
comptime fn as_tuple(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L169-L171" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L169-L171</a></sub></sup>


If this expression is a tuple `(field1, ..., fieldN)`,
return each element of the tuple.

### as_unary_op

```rust title="as_unary_op" showLineNumbers 
comptime fn as_unary_op(self) -> Option<(UnaryOp, Expr)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L176-L178" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L176-L178</a></sub></sup>


If this expression is a unary operation `<op> <rhs>`,
return the unary operator as well as the right-hand side expression.

### as_unsafe

```rust title="as_unsafe" showLineNumbers 
comptime fn as_unsafe(self) -> Option<[Expr]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L183-L185" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L183-L185</a></sub></sup>


If this expression is an `unsafe { stmt1; ...; stmtN }` block,
return each statement inside in a slice.

### has_semicolon

```rust title="has_semicolon" showLineNumbers 
comptime fn has_semicolon(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L204-L206" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L204-L206</a></sub></sup>


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
comptime fn is_break(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L210-L212" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L210-L212</a></sub></sup>


`true` if this expression is `break`.

### is_continue

```rust title="is_continue" showLineNumbers 
comptime fn is_continue(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L216-L218" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L216-L218</a></sub></sup>


`true` if this expression is `continue`.

### modify

```rust title="modify" showLineNumbers 
comptime fn modify<Env>(self, f: fn[Env](Expr) -> Option<Expr>) -> Expr {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L227-L229" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L227-L229</a></sub></sup>


Applies a mapping function to this expression and to all of its sub-expressions.
`f` will be applied to each sub-expression first, then applied to the expression itself.

This happens recursively for every expression within `self`.

For example, calling `modify` on `(&[1], &[2, 3])` with an `f` that returns `Option::some`
for expressions that are integers, doubling them, would return `(&[2], &[4, 6])`.

### quoted

```rust title="quoted" showLineNumbers 
comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L264-L266" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L264-L266</a></sub></sup>


Returns this expression as a `Quoted` value. It's the same as `quote { $self }`.

### resolve

```rust title="resolve" showLineNumbers 
comptime fn resolve(self, in_function: Option<FunctionDefinition>) -> TypedExpr {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/expr.nr#L280-L282" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/expr.nr#L280-L282</a></sub></sup>


Resolves and type-checks this expression and returns the result as a `TypedExpr`. 

The `in_function` argument specifies where the expression is resolved:
- If it's `none`, the expression is resolved in the function where `resolve` was called
- If it's `some`, the expression is resolved in the given function

If any names used by this expression are not in scope or if there are any type errors, 
this will give compiler errors as if the expression was written directly into 
the current `comptime` function.