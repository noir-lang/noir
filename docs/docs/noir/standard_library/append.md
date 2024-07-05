---
title: Append Trait
description:
  The Append Trait abstracts over types that can be appended to
keywords: [append, trait]
---

`Append` can abstract over types that can be appended to - usually container types:

```rs
trait Append {
    fn empty() -> Self;

    fn append(self, other: Self) -> Self;
}
```

`Append` requires two methods:

- `empty`: Constructs an empty value of `Self`.
- `append`: Append two values together, returning the result.

Additionally, it is expected that for any implementation:

- `T::empty().append(x) == x`
- `x.append(T::empty()) == x`

---

## Implementations

```rs
impl<T> Append for [T]
```

```rs
impl Append for Quoted
```
