use crate::cmp::{Eq, Ord, Ordering};
use crate::default::Default;
use crate::hash::{Hash, Hasher};

pub struct Option<T> {
    _is_some: bool,
    _value: T,
}

impl<T> Option<T> {
    /// Constructs a None value
    pub fn none() -> Self {
        Self { _is_some: false, _value: crate::mem::zeroed() }
    }

    /// Constructs a Some wrapper around the given value
    pub fn some(_value: T) -> Self {
        Self { _is_some: true, _value }
    }

    /// True if this Option is None
    pub fn is_none(self) -> bool {
        !self._is_some
    }

    /// True if this Option is Some
    pub fn is_some(self) -> bool {
        self._is_some
    }

    /// Asserts `self.is_some()` and returns the wrapped value.
    pub fn unwrap(self) -> T {
        assert(self._is_some);
        self._value
    }

    /// Returns the inner value without asserting `self.is_some()`
    /// Note that if `self` is `None`, there is no guarantee what value will be returned,
    /// only that it will be of type `T`.
    pub fn unwrap_unchecked(self) -> T {
        self._value
    }

    /// Returns the wrapped value if `self.is_some()`. Otherwise, returns the given default value.
    pub fn unwrap_or(self, default: T) -> T {
        if self._is_some {
            self._value
        } else {
            default
        }
    }

    /// Returns the wrapped value if `self.is_some()`. Otherwise, calls the given function to return
    /// a default value.
    pub fn unwrap_or_else<Env>(self, default: fn[Env]() -> T) -> T {
        if self._is_some {
            self._value
        } else {
            default()
        }
    }

    /// Asserts `self.is_some()` with a provided custom message and returns the contained `Some` value
    pub fn expect<let N: u32, MessageTypes>(self, message: fmtstr<N, MessageTypes>) -> T {
        assert(self.is_some(), message);
        self._value
    }

    /// If self is `Some(x)`, this returns `Some(f(x))`. Otherwise, this returns `None`.
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> Option<U> {
        if self._is_some {
            Option::some(f(self._value))
        } else {
            Option::none()
        }
    }

    /// If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns the given default value.
    pub fn map_or<U, Env>(self, default: U, f: fn[Env](T) -> U) -> U {
        if self._is_some {
            f(self._value)
        } else {
            default
        }
    }

    /// If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns `default()`.
    pub fn map_or_else<U, Env1, Env2>(self, default: fn[Env1]() -> U, f: fn[Env2](T) -> U) -> U {
        if self._is_some {
            f(self._value)
        } else {
            default()
        }
    }

    /// Returns None if self is None. Otherwise, this returns `other`.
    pub fn and(self, other: Self) -> Self {
        if self.is_none() {
            Option::none()
        } else {
            other
        }
    }

    /// If self is None, this returns None. Otherwise, this calls the given function
    /// with the Some value contained within self, and returns the result of that call.
    ///
    /// In some languages this function is called `flat_map` or `bind`.
    pub fn and_then<U, Env>(self, f: fn[Env](T) -> Option<U>) -> Option<U> {
        if self._is_some {
            f(self._value)
        } else {
            Option::none()
        }
    }

    /// If self is Some, return self. Otherwise, return `other`.
    pub fn or(self, other: Self) -> Self {
        if self._is_some {
            self
        } else {
            other
        }
    }

    /// If self is Some, return self. Otherwise, return `default()`.
    pub fn or_else<Env>(self, default: fn[Env]() -> Self) -> Self {
        if self._is_some {
            self
        } else {
            default()
        }
    }

    // If only one of the two Options is Some, return that option.
    // Otherwise, if both options are Some or both are None, None is returned.
    pub fn xor(self, other: Self) -> Self {
        if self._is_some {
            if other._is_some {
                Option::none()
            } else {
                self
            }
        } else if other._is_some {
            other
        } else {
            Option::none()
        }
    }

    /// Returns `Some(x)` if self is `Some(x)` and `predicate(x)` is true.
    /// Otherwise, this returns `None`
    pub fn filter<Env>(self, predicate: fn[Env](T) -> bool) -> Self {
        if self._is_some {
            if predicate(self._value) {
                self
            } else {
                Option::none()
            }
        } else {
            Option::none()
        }
    }

    /// Flattens an Option<Option<T>> into a Option<T>.
    /// This returns None if the outer Option is None. Otherwise, this returns the inner Option.
    pub fn flatten(option: Option<Option<T>>) -> Option<T> {
        if option._is_some {
            option._value
        } else {
            Option::none()
        }
    }
}

impl<T> Default for Option<T> {
    fn default() -> Self {
        Option::none()
    }
}

impl<T> Eq for Option<T>
where
    T: Eq,
{
    fn eq(self, other: Self) -> bool {
        if self._is_some == other._is_some {
            if self._is_some {
                self._value == other._value
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl<T> Hash for Option<T>
where
    T: Hash,
{
    fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        self._is_some.hash(state);
        if self._is_some {
            self._value.hash(state);
        }
    }
}

// For this impl we're declaring Option::none < Option::some
impl<T> Ord for Option<T>
where
    T: Ord,
{
    fn cmp(self, other: Self) -> Ordering {
        if self._is_some {
            if other._is_some {
                self._value.cmp(other._value)
            } else {
                Ordering::greater()
            }
        } else if other._is_some {
            Ordering::less()
        } else {
            Ordering::equal()
        }
    }
}
