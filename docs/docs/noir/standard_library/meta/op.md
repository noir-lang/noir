---
title: UnaryOp and BinaryOp
---

`std::meta::op` contains the `UnaryOp` and `BinaryOp` types as well as methods on them.
These types are used to represent a unary or binary operator respectively in Noir source code.

## Types

### UnaryOp

Represents a unary operator. One of `-`, `!`, `&mut`, or `*`.

### Methods

#### is_minus

#include_code is_minus noir_stdlib/src/meta/op.nr rust

Returns `true` if this operator is `-`.

#### is_not

#include_code is_not noir_stdlib/src/meta/op.nr rust

`true` if this operator is `!`

#### is_mutable_reference

#include_code is_mutable_reference noir_stdlib/src/meta/op.nr rust

`true` if this operator is `&mut`

#### is_dereference

#include_code is_dereference noir_stdlib/src/meta/op.nr rust

`true` if this operator is `*`

#### quoted

#include_code unary_quoted noir_stdlib/src/meta/op.nr rust

Returns this operator as a `Quoted` value.

### Trait Implementations

```rust
impl Eq for UnaryOp
impl Hash for UnaryOp
```

### BinaryOp

Represents a binary operator. One of `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&`, `|`, `^`, `>>`, or `<<`.

### Methods

#### is_add

#include_code is_add noir_stdlib/src/meta/op.nr rust

`true` if this operator is `+`

#### is_subtract

#include_code is_subtract noir_stdlib/src/meta/op.nr rust

`true` if this operator is `-`

#### is_multiply

#include_code is_multiply noir_stdlib/src/meta/op.nr rust

`true` if this operator is `*`

#### is_divide

#include_code is_divide noir_stdlib/src/meta/op.nr rust

`true` if this operator is `/`

#### is_modulo

#include_code is_modulo noir_stdlib/src/meta/op.nr rust

`true` if this operator is `%`

#### is_equal

#include_code is_equal noir_stdlib/src/meta/op.nr rust

`true` if this operator is `==`

#### is_not_equal

#include_code is_not_equal noir_stdlib/src/meta/op.nr rust

`true` if this operator is `!=`

#### is_less_than

#include_code is_less_than noir_stdlib/src/meta/op.nr rust

`true` if this operator is `<`

#### is_less_than_or_equal

#include_code is_less_than_or_equal noir_stdlib/src/meta/op.nr rust

`true` if this operator is `<=`

#### is_greater_than

#include_code is_greater_than noir_stdlib/src/meta/op.nr rust

`true` if this operator is `>`

#### is_greater_than_or_equal

#include_code is_greater_than_or_equal noir_stdlib/src/meta/op.nr rust

`true` if this operator is `>=`

#### is_and

#include_code is_and noir_stdlib/src/meta/op.nr rust

`true` if this operator is `&`

#### is_or

#include_code is_or noir_stdlib/src/meta/op.nr rust

`true` if this operator is `|`

#### is_shift_right

#include_code is_shift_right noir_stdlib/src/meta/op.nr rust

`true` if this operator is `>>`

#### is_shift_left

#include_code is_shift_left noir_stdlib/src/meta/op.nr rust

`true` if this operator is `<<`

#### quoted

#include_code binary_quoted noir_stdlib/src/meta/op.nr rust

Returns this operator as a `Quoted` value.

### Trait Implementations

```rust
impl Eq for BinaryOp
impl Hash for BinaryOp
```
