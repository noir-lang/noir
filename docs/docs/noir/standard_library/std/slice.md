---
title: slice
---

# Module `std::slice`

## `[T]` methods

### len

```noir
fn len(self) -> u32
```

Returns the length of the slice.

### push_back

```noir
fn push_back(self, elem: T) -> Self
```

Push a new element to the end of the slice, returning a
new slice with a length one greater than the
original unmodified slice.

### push_front

```noir
fn push_front(self, elem: T) -> Self
```

Push a new element to the front of the slice, returning a
new slice with a length one greater than the
original unmodified slice.

### pop_back

```noir
fn pop_back(self) -> (Self, T)
```

Remove the last element of the slice, returning the
popped slice and the element in a tuple

### pop_front

```noir
fn pop_front(self) -> (T, Self)
```

Remove the first element of the slice, returning the
element and the popped slice in a tuple

### insert

```noir
fn insert(self, index: u32, elem: T) -> Self
```

Insert an element at a specified index, shifting all elements
after it to the right

### remove

```noir
fn remove(self, index: u32) -> (Self, T)
```

Remove an element at a specified index, shifting all elements
after it to the left, returning the altered slice and
the removed element

### append

```noir
fn append(mut self, other: Self) -> Self
```

Append each element of the `other` slice to the end of `self`.
This returns a new slice and leaves both input slices unchanged.

### as_array

```noir
fn as_array<let N: u32>(self) -> [T; N]
```

### map

```noir
fn map<U, Env>(self, f: fn[Env](T) -> U) -> [U]
```

### fold

```noir
fn fold<U, Env>(self, mut accumulator: U, f: fn[Env](U, T) -> U) -> U
```

### reduce

```noir
fn reduce<Env>(self, f: fn[Env](T, T) -> T) -> T
```

### filter

```noir
fn filter<Env>(self, predicate: fn[Env](T) -> bool) -> Self
```

### join

```noir
fn join(self, separator: T) -> T
    where T: Append
```

### all

```noir
fn all<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

### any

```noir
fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

