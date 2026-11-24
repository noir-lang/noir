---
title: UnaryOp and BinaryOp
description: Represent and inspect operators at compile time—check kinds, and emit quoted operator tokens.
---

`std::meta::op` contains the `UnaryOp` and `BinaryOp` types as well as methods on them.
These types are used to represent a unary or binary operator respectively in Noir source code.

## Types

### UnaryOp

Represents a unary operator. One of `-`, `!`, `&mut`, or `*`.

### Methods

#### is_minus

```rust title="is_minus" showLineNumbers 
pub fn is_minus(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L53-L55" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L53-L55</a></sub></sup>


Returns `true` if this operator is `-`.

#### is_not

```rust title="is_not" showLineNumbers 
pub fn is_not(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L72-L74" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L72-L74</a></sub></sup>


`true` if this operator is `!`

#### is_mutable_reference

```rust title="is_mutable_reference" showLineNumbers 
pub fn is_mutable_reference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L91-L93" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L91-L93</a></sub></sup>


`true` if this operator is `&mut`

#### is_dereference

```rust title="is_dereference" showLineNumbers 
pub fn is_dereference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L110-L112" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L110-L112</a></sub></sup>


`true` if this operator is `*`

#### quoted

```rust title="unary_quoted" showLineNumbers 
pub comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L130-L132" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L130-L132</a></sub></sup>


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

```rust title="is_add" showLineNumbers 
pub fn is_add(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L195-L197" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L195-L197</a></sub></sup>


`true` if this operator is `+`

#### is_subtract

```rust title="is_subtract" showLineNumbers 
pub fn is_subtract(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L214-L216" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L214-L216</a></sub></sup>


`true` if this operator is `-`

#### is_multiply

```rust title="is_multiply" showLineNumbers 
pub fn is_multiply(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L233-L235" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L233-L235</a></sub></sup>


`true` if this operator is `*`

#### is_divide

```rust title="is_divide" showLineNumbers 
pub fn is_divide(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L252-L254" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L252-L254</a></sub></sup>


`true` if this operator is `/`

#### is_modulo

```rust title="is_modulo" showLineNumbers 
pub fn is_modulo(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L480-L482" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L480-L482</a></sub></sup>


`true` if this operator is `%`

#### is_equal

```rust title="is_equal" showLineNumbers 
pub fn is_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L271-L273" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L271-L273</a></sub></sup>


`true` if this operator is `==`

#### is_not_equal

```rust title="is_not_equal" showLineNumbers 
pub fn is_not_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L290-L292" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L290-L292</a></sub></sup>


`true` if this operator is `!=`

#### is_less_than

```rust title="is_less_than" showLineNumbers 
pub fn is_less_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L309-L311" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L309-L311</a></sub></sup>


`true` if this operator is `<`

#### is_less_than_or_equal

```rust title="is_less_than_or_equal" showLineNumbers 
pub fn is_less_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L328-L330" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L328-L330</a></sub></sup>


`true` if this operator is `<=`

#### is_greater_than

```rust title="is_greater_than" showLineNumbers 
pub fn is_greater_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L347-L349" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L347-L349</a></sub></sup>


`true` if this operator is `>`

#### is_greater_than_or_equal

```rust title="is_greater_than_or_equal" showLineNumbers 
pub fn is_greater_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L366-L368" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L366-L368</a></sub></sup>


`true` if this operator is `>=`

#### is_and

```rust title="is_and" showLineNumbers 
pub fn is_and(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L385-L387" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L385-L387</a></sub></sup>


`true` if this operator is `&`

#### is_or

```rust title="is_or" showLineNumbers 
pub fn is_or(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L404-L406" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L404-L406</a></sub></sup>


`true` if this operator is `|`

#### is_shift_right

```rust title="is_shift_right" showLineNumbers 
pub fn is_shift_right(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L442-L444" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L442-L444</a></sub></sup>


`true` if this operator is `>>`

#### is_shift_left

```rust title="is_shift_left" showLineNumbers 
pub fn is_shift_left(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L461-L463" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L461-L463</a></sub></sup>


`true` if this operator is `<<`

#### quoted

```rust title="binary_quoted" showLineNumbers 
pub comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L500-L502" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L500-L502</a></sub></sup>


Returns this operator as a `Quoted` value.

### Trait Implementations

```rust
impl Eq for BinaryOp
impl Hash for BinaryOp
```
