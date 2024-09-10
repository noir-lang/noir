---
title: umap
---

# Module `std::collections::umap`

### Methods

#### with_hasher

```noir
fn with_hasher<H>(_build_hasher: B) -> Self
    where B: BuildHasher<H>
```

#### with_hasher_and_capacity

```noir
fn with_hasher_and_capacity<H>(_build_hasher: B, capacity: u32) -> Self
    where B: BuildHasher<H>
```

#### clear

```noir
fn clear(self)
```

#### contains_key

```noir
fn contains_key<H>(self, key: K) -> bool
    where K: Hash,
          K: Eq,
          B: BuildHasher<H>,
          H: Hasher
```

#### is_empty

```noir
fn is_empty(self) -> bool
```

#### entries

```noir
fn entries(self) -> [(K, V)]
```

#### keys

```noir
fn keys(self) -> [K]
```

#### values

```noir
fn values(self) -> [V]
```

#### iter_mut

```noir
fn iter_mut<H>(self, f: fn(K, V) -> (K, V))
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

#### iter_keys_mut

```noir
fn iter_keys_mut<H>(self, f: fn(K) -> K)
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

#### iter_values_mut

```noir
fn iter_values_mut(self, f: fn(V) -> V)
```

#### retain

```noir
fn retain(self, f: fn(K, V) -> bool)
```

#### len

```noir
fn len(self) -> u32
```

#### capacity

```noir
fn capacity(self) -> u32
```

#### get

```noir
fn get<H>(self, key: K) -> Option<V>
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

#### insert

```noir
fn insert<H>(self, key: K, value: V)
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

