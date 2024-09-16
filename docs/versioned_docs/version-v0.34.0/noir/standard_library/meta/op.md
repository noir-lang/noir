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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L7-L9" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L7-L9</a></sub></sup>


Returns `true` if this operator is `-`.

#### is_not

```rust title="is_not" showLineNumbers 
pub fn is_not(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L13-L15" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L13-L15</a></sub></sup>


`true` if this operator is `!`

#### is_mutable_reference

```rust title="is_mutable_reference" showLineNumbers 
pub fn is_mutable_reference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L19-L21" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L19-L21</a></sub></sup>


`true` if this operator is `&mut`

#### is_dereference

```rust title="is_dereference" showLineNumbers 
pub fn is_dereference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L25-L27" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L25-L27</a></sub></sup>


`true` if this operator is `*`

#### quoted

```rust title="unary_quoted" showLineNumbers 
pub comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L31-L33" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L31-L33</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L55-L57" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L55-L57</a></sub></sup>


`true` if this operator is `+`

#### is_subtract

```rust title="is_subtract" showLineNumbers 
pub fn is_subtract(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L61-L63" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L61-L63</a></sub></sup>


`true` if this operator is `-`

#### is_multiply

```rust title="is_multiply" showLineNumbers 
pub fn is_multiply(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L67-L69" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L67-L69</a></sub></sup>


`true` if this operator is `*`

#### is_divide

```rust title="is_divide" showLineNumbers 
pub fn is_divide(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L73-L75" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L73-L75</a></sub></sup>


`true` if this operator is `/`

#### is_modulo

```rust title="is_modulo" showLineNumbers 
pub fn is_modulo(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L145-L147" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L145-L147</a></sub></sup>


`true` if this operator is `%`

#### is_equal

```rust title="is_equal" showLineNumbers 
pub fn is_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L79-L81" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L79-L81</a></sub></sup>


`true` if this operator is `==`

#### is_not_equal

```rust title="is_not_equal" showLineNumbers 
pub fn is_not_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L85-L87" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L85-L87</a></sub></sup>


`true` if this operator is `!=`

#### is_less_than

```rust title="is_less_than" showLineNumbers 
pub fn is_less_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L91-L93" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L91-L93</a></sub></sup>


`true` if this operator is `<`

#### is_less_than_or_equal

```rust title="is_less_than_or_equal" showLineNumbers 
pub fn is_less_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L97-L99" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L97-L99</a></sub></sup>


`true` if this operator is `<=`

#### is_greater_than

```rust title="is_greater_than" showLineNumbers 
pub fn is_greater_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L103-L105" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L103-L105</a></sub></sup>


`true` if this operator is `>`

#### is_greater_than_or_equal

```rust title="is_greater_than_or_equal" showLineNumbers 
pub fn is_greater_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L109-L111" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L109-L111</a></sub></sup>


`true` if this operator is `>=`

#### is_and

```rust title="is_and" showLineNumbers 
pub fn is_and(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L115-L117" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L115-L117</a></sub></sup>


`true` if this operator is `&`

#### is_or

```rust title="is_or" showLineNumbers 
pub fn is_or(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L121-L123" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L121-L123</a></sub></sup>


`true` if this operator is `|`

#### is_shift_right

```rust title="is_shift_right" showLineNumbers 
pub fn is_shift_right(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L133-L135" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L133-L135</a></sub></sup>


`true` if this operator is `>>`

#### is_shift_left

```rust title="is_shift_right" showLineNumbers 
pub fn is_shift_right(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L133-L135" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L133-L135</a></sub></sup>


`true` if this operator is `<<`

#### quoted

```rust title="binary_quoted" showLineNumbers 
pub comptime fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L151-L153" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L151-L153</a></sub></sup>


Returns this operator as a `Quoted` value.

### Trait Implementations

```rust
impl Eq for BinaryOp
impl Hash for BinaryOp
```
