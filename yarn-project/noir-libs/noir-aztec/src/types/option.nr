// TODO: this will soon be a part of the Noir stdlib, so we can remove this file, eventually: https://github.com/noir-lang/noir/pull/1781

struct Option<T> {
    _is_some: bool,
    _value: T,
}

impl<T> Option<T> {
    // We'll be wrapping custom structs in the Option type, so we don't want to use the unsafe `zeroed` function. Instead, we'll use constructors which allow the caller to hint at what an 'empty' `T` is.

    // fn none() -> Self {
    //     Self { _is_some: false, value: crate::unsafe::zeroed() }
    // }

    // fn some(value: T) -> Self {
    //     Self { _is_some: true, value }
    // }

    fn none(none_value: T) -> Self {
        Option {
            _is_some: false,
            _value: none_value,
        }
    }

    fn some(value: T) -> Self {
        Option {
            _is_some: true,
            _value: value,
        }
    }

    fn is_none(self) -> bool {
        !self._is_some
    }

    fn is_some(self) -> bool {
        self._is_some
    }

    // Returns the contained Some value, consuming the self value, without checking that the value is not None.
    // Safety:
    // Calling this method on None is undefined behavior.
    fn unwrap_unchecked(self) -> T {
        self._value
    }

    fn unwrap(self) -> T {
        assert(self._is_some);
        self._value
    }

    fn unwrap_or(self, default: T) -> T {
        if self._is_some {
            self._value
        } else {
            default
        }
    }

    fn unwrap_or_else(self, default: fn() -> T) -> T {
        if self._is_some {
            self._value
        } else {
            default()
        }
    }

    // Maps an Option<T> to Option<U> by applying a function to a contained value (if Some) or returns None (if None).
    fn map<U>(self, f: fn(T) -> U, none_u: U) -> Option<U> {
        if self._is_some {
            Option::some(f(self._value))
        } else {
            Option::none(none_u)
        }
    }

    // Returns the provided default result (if none), or applies a function to the contained value (if any).
    fn map_or<U>(self, default: U, f: fn(T) -> U) -> U {
        if self._is_some {
            f(self._value)
        } else {
            default
        }
    }

    // Computes a default function result (if none), or applies a different function to the contained value (if any).
    fn map_or_else<U>(self, default: fn() -> U, f: fn(T) -> U) -> U {
        if self._is_some {
            f(self._value)
        } else {
            default()
        }
    }

    // Returns None if the option is None, otherwise returns other.
    fn and(self, other: Self) -> Self {
        if self.is_none() {
            // Option::none()
            self
        } else {
            other
        }
    }

    // Returns None if the option is None, otherwise calls f with the wrapped value and returns the result.
    fn and_then<U>(self, f: fn(T) -> Option<U>, none_u: U) -> Option<U> {
        if self._is_some {
            f(self._value)
        } else {
            Option::none(none_u)
        }
    }

    // Returns the option if it contains a value, otherwise returns other.
    fn or(self, other: Self) -> Self {
        if self._is_some {
            self
        } else {
            other
        }
    }

    // Returns the option if it contains a value, otherwise calls f and returns the result.
    fn or_else<U>(self, f: fn() -> Option<T>) -> Option<T> {
        if self._is_some {
            self
        } else {
            f()
        }
    }

    fn xor(self, other: Self, none_value: T) -> Self {
        if self._is_some {
            if other._is_some {
                Option::none(none_value)
            } else {
                self
            }
        } else if other._is_some {
            other
        } else {
            // Option::none()
            self
        }
    }
}

#[test]
fn test_option() {
    let none = Option::none(0);
    let some = Option::some(3);

    assert(none.is_none());
    assert(some.is_some());

    assert(some.unwrap() == 3);

    assert(none.unwrap_or(2) == 2);
    assert(some.unwrap_or(2) == 3);

    assert(none.unwrap_or_else(|| 5) == 5);
    assert(some.unwrap_or_else(|| 5) == 3);

    assert(none.map(|x| x * 2, 0).is_none());
    assert(some.map(|x| x * 2, 0).unwrap() == 6);

    assert(none.map_or(0, |x| x * 2) == 0);
    assert(some.map_or(0, |x| x * 2) == 6);

    assert(none.map_or_else(|| 0, |x| x * 2) == 0);
    assert(some.map_or_else(|| 0, |x| x * 2) == 6);

    assert(none.and(none).is_none());
    assert(none.and(some).is_none());
    assert(some.and(none).is_none());
    assert(some.and(some).is_some());

    let add1_u64 = |value: Field| Option::some(value as u64 + 1);

    assert(none.and_then(|_value| Option::none(0), 0).is_none());
    assert(none.and_then(add1_u64, 0).is_none());
    assert(some.and_then(|_value| Option::none(0), 0).is_none());
    assert(some.and_then(add1_u64, 0).unwrap() == 4);

    assert(none.or(none).is_none());
    assert(none.or(some).is_some());
    assert(some.or(none).is_some());
    assert(some.or(some).is_some());

    assert(none.or_else(|| Option::none(0)).is_none());
    assert(none.or_else(|| Option::some(5)).is_some());
    assert(some.or_else(|| Option::none(0)).is_some());
    assert(some.or_else(|| Option::some(5)).is_some());

    assert(none.xor(none, 0).is_none());
    assert(none.xor(some, 0).is_some());
    assert(some.xor(none, 0).is_some());
    assert(some.xor(some, 0).is_none());
}