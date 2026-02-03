---
title: Option<T> Type
---

The `Option<T>` type is a way to express that a value might be present (`Some(T))` or absent (`None`). It's a safer way to handle potential absence of values, compared to using nulls in many other languages.

```rust
struct Option<T> {
    None,
    Some(T),
}
```

The `Option` type, already imported into your Noir program, can be used directly:

```rust
fn main() {
    let none = Option::none();
    let some = Option::some(3);
}
```

See [this test](https://github.com/noir-lang/noir/blob/5cbfb9c4a06c8865c98ff2b594464b037d821a5c/crates/nargo_cli/tests/test_data/option/src/main.nr) for a more comprehensive set of examples of each of the methods described below.

## Methods

### none

Constructs a none value.

### some

Constructs a some wrapper around a given value.

### is_none

Returns true if the Option is None.

### is_some

Returns true of the Option is Some.

### unwrap

Asserts `self.is_some()` and returns the wrapped value.

### unwrap_unchecked

Returns the inner value without asserting `self.is_some()`. This method can be useful within an if condition when we already know that `option.is_some()`. If the option is None, there is no guarantee what value will be returned, only that it will be of type T for an `Option<T>`.

### unwrap_or

Returns the wrapped value if `self.is_some()`. Otherwise, returns the given default value.

### unwrap_or_else

Returns the wrapped value if `self.is_some()`. Otherwise, calls the given function to return a default value.

### expect

Asserts `self.is_some()` with a provided custom message and returns the contained `Some` value. The custom message is expected to be a format string.

### map

If self is `Some(x)`, this returns `Some(f(x))`. Otherwise, this returns `None`.

### map_or

If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns the given default value.

### map_or_else

If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns `default()`.

### and

Returns None if self is None. Otherwise, this returns `other`.

### and_then

If self is None, this returns None. Otherwise, this calls the given function with the Some value contained within self, and returns the result of that call. In some languages this function is called `flat_map` or `bind`.

### or

If self is Some, return self. Otherwise, return `other`.

### or_else

If self is Some, return self. Otherwise, return `default()`.

### xor

If only one of the two Options is Some, return that option. Otherwise, if both options are Some or both are None, None is returned.

### filter

Returns `Some(x)` if self is `Some(x)` and `predicate(x)` is true. Otherwise, this returns `None`.

### flatten

Flattens an `Option<Option<T>>` into a `Option<T>`. This returns `None` if the outer Option is None. Otherwise, this returns the inner Option.
