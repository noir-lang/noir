---
title: Expr
---

`std::meta::expr` contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

## Methods

### as_array

#include_code as_array noir_stdlib/src/meta/expr.nr rust

If this expression is an array, this returns a slice of each element in the array.

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

### as_comptime

#include_code as_comptime noir_stdlib/src/meta/expr.nr rust

If this expression is a `comptime { stmt1; stmt2; ...; stmtN }` block,
return each statement in the block.

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

If this element is an integer literal, return the integer as a field
as well as whether the integer is negative (true) or not (false).

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
