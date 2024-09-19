---
title: Expr
---

`std::meta::expr` contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

## Methods

### as_array

#include_code as_array noir_stdlib/src/meta/expr.nr rust

If this expression is an array, this returns a slice of each element in the array.

### as_assert

#include_code as_assert noir_stdlib/src/meta/expr.nr rust

If this expression is an assert, this returns the assert expression and the optional message.

### as_assert_eq

#include_code as_assert_eq noir_stdlib/src/meta/expr.nr rust

If this expression is an assert_eq, this returns the left-hand-side and right-hand-side
expressions, together with the optional message.

### as_assign

#include_code as_assign noir_stdlib/src/meta/expr.nr rust

If this expression is an assignment, this returns a tuple with the left hand side
and right hand side in order.

### as_binary_op

#include_code as_binary_op noir_stdlib/src/meta/expr.nr rust

If this expression is a binary operator operation `<lhs> <op> <rhs>`,
return the left-hand side, operator, and the right-hand side of the operation.

### as_block

#include_code as_block noir_stdlib/src/meta/expr.nr rust

If this expression is a block `{ stmt1; stmt2; ...; stmtN }`, return
a slice containing each statement.

### as_bool

#include_code as_bool noir_stdlib/src/meta/expr.nr rust

If this expression is a boolean literal, return that literal.

### as_cast

#include_code as_cast noir_stdlib/src/meta/expr.nr rust

If this expression is a cast expression (`expr as type`), returns the casted
expression and the type to cast to.

### as_comptime

#include_code as_comptime noir_stdlib/src/meta/expr.nr rust

If this expression is a `comptime { stmt1; stmt2; ...; stmtN }` block,
return each statement in the block.

### as_constructor

#include_code as_constructor noir_stdlib/src/meta/expr.nr rust

If this expression is a constructor `Type { field1: expr1, ..., fieldN: exprN }`,
return the type and the fields.

### as_for

#include_code as_for noir_stdlib/src/meta/expr.nr rust

If this expression is a for statement over a single expression, return the identifier,
the expression and the for loop body.

### as_for_range

#include_code as_for noir_stdlib/src/meta/expr.nr rust

If this expression is a for statement over a range, return the identifier,
the range start, the range end and the for loop body.

### as_function_call

#include_code as_function_call noir_stdlib/src/meta/expr.nr rust

If this expression is a function call `foo(arg1, ..., argN)`, return
the function and a slice of each argument.

### as_if

#include_code as_if noir_stdlib/src/meta/expr.nr rust

If this expression is an `if condition { then_branch } else { else_branch }`,
return the condition, then branch, and else branch. If there is no else branch,
`None` is returned for that branch instead.

### as_index

#include_code as_index noir_stdlib/src/meta/expr.nr rust

If this expression is an index into an array `array[index]`, return the
array and the index.

### as_integer

#include_code as_integer noir_stdlib/src/meta/expr.nr rust

If this expression is an integer literal, return the integer as a field
as well as whether the integer is negative (true) or not (false).

### as_lambda

#include_code as_lambda noir_stdlib/src/meta/expr.nr rust

If this expression is a lambda, returns the parameters, return type and body.

### as_let

#include_code as_let noir_stdlib/src/meta/expr.nr rust

If this expression is a let statement, returns the let pattern as an `Expr`,
the optional type annotation, and the assigned expression.

### as_member_access

#include_code as_member_access noir_stdlib/src/meta/expr.nr rust

If this expression is a member access `foo.bar`, return the struct/tuple
expression and the field. The field will be represented as a quoted value.

### as_method_call

#include_code as_method_call noir_stdlib/src/meta/expr.nr rust

If this expression is a method call `foo.bar::<generic1, ..., genericM>(arg1, ..., argN)`, return
the receiver, method name, a slice of each generic argument, and a slice of each argument.

### as_repeated_element_array

#include_code as_repeated_element_array noir_stdlib/src/meta/expr.nr rust

If this expression is a repeated element array `[elem; length]`, return
the repeated element and the length expressions.

### as_repeated_element_slice

#include_code as_repeated_element_slice noir_stdlib/src/meta/expr.nr rust

If this expression is a repeated element slice `[elem; length]`, return
the repeated element and the length expressions.

### as_slice

#include_code as_slice noir_stdlib/src/meta/expr.nr rust

If this expression is a slice literal `&[elem1, ..., elemN]`,
return each element of the slice.

### as_tuple

#include_code as_tuple noir_stdlib/src/meta/expr.nr rust

If this expression is a tuple `(field1, ..., fieldN)`,
return each element of the tuple.

### as_unary_op

#include_code as_unary_op noir_stdlib/src/meta/expr.nr rust

If this expression is a unary operation `<op> <rhs>`,
return the unary operator as well as the right-hand side expression.

### as_unsafe

#include_code as_unsafe noir_stdlib/src/meta/expr.nr rust

If this expression is an `unsafe { stmt1; ...; stmtN }` block,
return each statement inside in a slice.

### has_semicolon

#include_code has_semicolon noir_stdlib/src/meta/expr.nr rust

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

#include_code is_break noir_stdlib/src/meta/expr.nr rust

`true` if this expression is `break`.

### is_continue

#include_code is_continue noir_stdlib/src/meta/expr.nr rust

`true` if this expression is `continue`.

### modify

#include_code modify noir_stdlib/src/meta/expr.nr rust

Applies a mapping function to this expression and to all of its sub-expressions.
`f` will be applied to each sub-expression first, then applied to the expression itself.

This happens recursively for every expression within `self`.

For example, calling `modify` on `(&[1], &[2, 3])` with an `f` that returns `Option::some`
for expressions that are integers, doubling them, would return `(&[2], &[4, 6])`.

### quoted

#include_code quoted noir_stdlib/src/meta/expr.nr rust

Returns this expression as a `Quoted` value. It's the same as `quote { $self }`.

### resolve

#include_code resolve noir_stdlib/src/meta/expr.nr rust

Resolves and type-checks this expression and returns the result as a `TypedExpr`. 

The `in_function` argument specifies where the expression is resolved:
- If it's `none`, the expression is resolved in the function where `resolve` was called
- If it's `some`, the expression is resolved in the given function

If any names used by this expression are not in scope or if there are any type errors, 
this will give compiler errors as if the expression was written directly into 
the current `comptime` function.