---
title: slice
---

# Module `std::slice`

## `[T]` methods

### len

```rust
fn len(self) -> u32
```

Returns the length of the slice.

### push_back

```rust
fn push_back(self, elem: T) -> Self
```

Push a new element to the end of the slice, returning a
new slice with a length one greater than the
original unmodified slice.

### push_front

```rust
fn push_front(self, elem: T) -> Self
```

Push a new element to the front of the slice, returning a
new slice with a length one greater than the
original unmodified slice.

### pop_back

```rust
fn pop_back(self) -> (Self, T)
```

Remove the last element of the slice, returning the
popped slice and the element in a tuple

### pop_front

```rust
fn pop_front(self) -> (T, Self)
```

Remove the first element of the slice, returning the
element and the popped slice in a tuple

### insert

```rust
fn insert(self, index: u32, elem: T) -> Self
```

Insert an element at a specified index, shifting all elements
after it to the right

### remove

```rust
fn remove(self, index: u32) -> (Self, T)
```

Remove an element at a specified index, shifting all elements
after it to the left, returning the altered slice and
the removed element

### append

```rust
fn append(mut self, other: Self) -> Self
```

Append each element of the `other` slice to the end of `self`.
This returns a new slice and leaves both input slices unchanged.

### as_array

```rust
fn as_array<let N: u32>(self) -> [T; N]
```

### map

```rust
fn map<U, Env>(self, f: fn[Env](T) -> U) -> [U]
```

### fold

```rust
fn fold<U, Env>(self, mut accumulator: U, f: fn[Env](U, T) -> U) -> U
```

### reduce

```rust
fn reduce<Env>(self, f: fn[Env](T, T) -> T) -> T
```

### filter

```rust
fn filter<Env>(self, predicate: fn[Env](T) -> bool) -> Self
```

### join

```rust
fn join(self, separator: T) -> T
    where T: Append
```

### all

```rust
fn all<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

### any

```rust
fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

