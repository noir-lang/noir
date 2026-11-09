---
title: ItemVisibility
description: Inspect item visibility at compile time and emit it as quoted syntax.
---

`std::meta::item_visibility` contains the `ItemVisibility` type, which represents whether an
item is private, public within its crate, or public outside its crate.

## Methods

### is_private

```rust title="is_private" showLineNumbers 
pub comptime fn is_private(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/item_visibility.nr#L50-L52" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/item_visibility.nr#L50-L52</a></sub></sup>


Returns `true` if the item is private.

### is_public

```rust title="is_public" showLineNumbers 
pub comptime fn is_public(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/item_visibility.nr#L75-L77" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/item_visibility.nr#L75-L77</a></sub></sup>


Returns `true` if the item is public outside its crate.

### is_public_crate

```rust title="is_public_crate" showLineNumbers 
pub comptime fn is_public_crate(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/item_visibility.nr#L99-L101" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/item_visibility.nr#L99-L101</a></sub></sup>


Returns `true` if the item is public only within its crate (`pub(crate)`).

### as_quoted

```rust title="as_quoted" showLineNumbers 
pub comptime fn as_quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/item_visibility.nr#L126-L128" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/item_visibility.nr#L126-L128</a></sub></sup>


Returns the visibility as a `Quoted` value for use in generated code. The result is `quote {}`
for private visibility, `quote { pub(crate) }` for crate visibility, or `quote { pub }` for
public visibility.
