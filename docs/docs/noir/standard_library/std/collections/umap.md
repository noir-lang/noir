---
title: umap
---

# Module `std::collections::umap`

### Methods

#### with_hasher

```rust
fn with_hasher<H>(_build_hasher: B) -> Self
    where B: BuildHasher<H>
```

#### with_hasher_and_capacity

```rust
fn with_hasher_and_capacity<H>(_build_hasher: B, capacity: u32) -> Self
    where B: BuildHasher<H>
```

#### clear

```rust
fn clear(self)
```

#### contains_key

```rust
fn contains_key<H>(self, key: K) -> bool
    where K: Hash,
          K: Eq,
          B: BuildHasher<H>,
          H: Hasher
```

#### is_empty

```rust
fn is_empty(self) -> bool
```

#### entries

```rust
fn entries(self) -> [(K, V)]
```

#### keys

```rust
fn keys(self) -> [K]
```

#### values

```rust
fn values(self) -> [V]
```

#### iter_mut

```rust
fn iter_mut<H>(self, f: fn(K, V) -> (K, V))
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

#### iter_keys_mut

```rust
fn iter_keys_mut<H>(self, f: fn(K) -> K)
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

#### iter_values_mut

```rust
fn iter_values_mut(self, f: fn(V) -> V)
```

#### retain

```rust
fn retain(self, f: fn(K, V) -> bool)
```

#### len

```rust
fn len(self) -> u32
```

#### capacity

```rust
fn capacity(self) -> u32
```

#### get

```rust
fn get<H>(self, key: K) -> Option<V>
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

#### insert

```rust
fn insert<H>(self, key: K, value: V)
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

