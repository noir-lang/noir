# Module `std::array`

## `[T; N]` methods

### len

```noir
fn len(self) -> u32
```

Returns the length of the slice.

### sort

```noir
fn sort(self) -> Self
    where T: Ord
```

### sort_via

```noir
fn sort_via<Env>(self, ordering: fn[Env](T, T) -> bool) -> Self
```

### get_sorting_index

```noir
fn get_sorting_index<Env>(self, ordering: fn[Env](T, T) -> bool) -> [u32; N]
```

Returns the index of the elements in the array that would sort it, using the provided custom sorting function.

### as_slice

```noir
fn as_slice(self) -> [T]
```

### map

```noir
fn map<U, Env>(self, f: fn[Env](T) -> U) -> [U; N]
```

### fold

```noir
fn fold<U, Env>(self, mut accumulator: U, f: fn[Env](U, T) -> U) -> U
```

### reduce

```noir
fn reduce<Env>(self, f: fn[Env](T, T) -> T) -> T
```

### all

```noir
fn all<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

### any

```noir
fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool
```

## `[u8; N]` methods

### as_str_unchecked

```noir
fn as_str_unchecked(self) -> str<N>
```

Convert a sequence of bytes as-is into a string.
This function performs no UTF-8 validation or similar.

