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

```rust title="is_minus" showLineNumbers 
pub fn is_minus(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L21-L23" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L21-L23</a></sub></sup>


Returns `true` if this operator is `-`.

#### is_not

```rust title="is_not" showLineNumbers 
pub fn is_not(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L27-L29" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L27-L29</a></sub></sup>


`true` if this operator is `!`

#### is_mutable_reference

```rust title="is_mutable_reference" showLineNumbers 
pub fn is_mutable_reference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L33-L35" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L33-L35</a></sub></sup>


`true` if this operator is `&mut`

#### is_dereference

```rust title="is_dereference" showLineNumbers 
pub fn is_dereference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L39-L41" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L39-L41</a></sub></sup>


`true` if this operator is `*`

#### quoted

```rust title="unary_quoted" showLineNumbers 
pub comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L45-L47" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L45-L47</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L80-L82" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L80-L82</a></sub></sup>


`true` if this operator is `+`

#### is_subtract

```rust title="is_subtract" showLineNumbers 
pub fn is_subtract(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L86-L88" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L86-L88</a></sub></sup>


`true` if this operator is `-`

#### is_multiply

```rust title="is_multiply" showLineNumbers 
pub fn is_multiply(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L92-L94" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L92-L94</a></sub></sup>


`true` if this operator is `*`

#### is_divide

```rust title="is_divide" showLineNumbers 
pub fn is_divide(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L98-L100" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L98-L100</a></sub></sup>


`true` if this operator is `/`

#### is_modulo

```rust title="is_modulo" showLineNumbers 
pub fn is_modulo(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L170-L172" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L170-L172</a></sub></sup>


`true` if this operator is `%`

#### is_equal

```rust title="is_equal" showLineNumbers 
pub fn is_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L104-L106" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L104-L106</a></sub></sup>


`true` if this operator is `==`

#### is_not_equal

```rust title="is_not_equal" showLineNumbers 
pub fn is_not_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L110-L112" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L110-L112</a></sub></sup>


`true` if this operator is `!=`

#### is_less_than

```rust title="is_less_than" showLineNumbers 
pub fn is_less_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L116-L118" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L116-L118</a></sub></sup>


`true` if this operator is `<`

#### is_less_than_or_equal

```rust title="is_less_than_or_equal" showLineNumbers 
pub fn is_less_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L122-L124" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L122-L124</a></sub></sup>


`true` if this operator is `<=`

#### is_greater_than

```rust title="is_greater_than" showLineNumbers 
pub fn is_greater_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L128-L130" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L128-L130</a></sub></sup>


`true` if this operator is `>`

#### is_greater_than_or_equal

```rust title="is_greater_than_or_equal" showLineNumbers 
pub fn is_greater_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L134-L136" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L134-L136</a></sub></sup>


`true` if this operator is `>=`

#### is_and

```rust title="is_and" showLineNumbers 
pub fn is_and(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L140-L142" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L140-L142</a></sub></sup>


`true` if this operator is `&`

#### is_or

```rust title="is_or" showLineNumbers 
pub fn is_or(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L146-L148" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L146-L148</a></sub></sup>


`true` if this operator is `|`

#### is_shift_right

```rust title="is_shift_right" showLineNumbers 
pub fn is_shift_right(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L158-L160" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L158-L160</a></sub></sup>


`true` if this operator is `>>`

#### is_shift_left

```rust title="is_shift_right" showLineNumbers 
pub fn is_shift_right(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L158-L160" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L158-L160</a></sub></sup>


`true` if this operator is `<<`

#### quoted

```rust title="binary_quoted" showLineNumbers 
pub comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L176-L178" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L176-L178</a></sub></sup>


Returns this operator as a `Quoted` value.

### Trait Implementations

```rust
impl Eq for BinaryOp
impl Hash for BinaryOp
```
