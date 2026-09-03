---
title: ItemVisibility
description: Inspect item visibility at compile time and emit it as quoted syntax.
---

`std::meta::item_visibility` contains the `ItemVisibility` type, which represents whether an
item is private, public within its crate, or public outside its crate.

## Methods

### is_private

#include_code is_private noir_stdlib/src/meta/item_visibility.nr rust

Returns `true` if the item is private.

### is_public

#include_code is_public noir_stdlib/src/meta/item_visibility.nr rust

Returns `true` if the item is public outside its crate.

### is_public_crate

#include_code is_public_crate noir_stdlib/src/meta/item_visibility.nr rust

Returns `true` if the item is public only within its crate (`pub(crate)`).

### as_quoted

#include_code as_quoted noir_stdlib/src/meta/item_visibility.nr rust

Returns the visibility as a `Quoted` value for use in generated code. The result is `quote {}`
for private visibility, `quote { pub(crate) }` for crate visibility, or `quote { pub }` for
public visibility.
