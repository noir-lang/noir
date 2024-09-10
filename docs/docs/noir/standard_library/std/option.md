---
title: option
---

# Module `std::option`

### Methods

#### none

```noir
fn none() -> Self
```

Constructs a None value

#### some

```noir
fn some(_value: T) -> Self
```

Constructs a Some wrapper around the given value

#### is_none

```noir
fn is_none(self) -> bool
```

True if this Option is None

#### is_some

```noir
fn is_some(self) -> bool
```

True if this Option is Some

#### unwrap

```noir
fn unwrap(self) -> T
```

Asserts `self.is_some()` and returns the wrapped value.

#### unwrap_unchecked

```noir
fn unwrap_unchecked(self) -> T
```

Returns the inner value without asserting `self.is_some()`
Note that if `self` is `None`, there is no guarantee what value will be returned,
only that it will be of type `T`.

#### unwrap_or

```noir
fn unwrap_or(self, default: T) -> T
```

Returns the wrapped value if `self.is_some()`. Otherwise, returns the given default value.

#### unwrap_or_else

```noir
fn unwrap_or_else<Env>(self, default: fn[Env]() -> T) -> T
```

Returns the wrapped value if `self.is_some()`. Otherwise, calls the given function to return
a default value.

