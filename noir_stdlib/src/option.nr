use crate::cmp::{Eq, Ord, Ordering};
use crate::default::Default;
use crate::hash::{Hash, Hasher};

/// Represents a value of type T or its absence.
/// Use `Option::some(value)` to construct a value or `Option::none()` to record the absence of one.
///
/// `Option` is in the prelude. A `None` still holds a value of type `T` internally, and only
/// [`Option::unwrap_unchecked`] exposes it. `Option::none()` sets it to the zeroed value of `T`,
/// but nothing constrains it in general: an `Option` returned by unconstrained code or passed in as
/// a program input can hold any value in a `None`. `Option` implements [`Eq`](crate::cmp::Eq),
/// [`Ord`](crate::cmp::Ord) (with `None` less than any `Some`), [`Hash`](crate::hash::Hash) and
/// [`Default`](crate::default::Default) (returning `None`).
///
/// Example:
///
/// ```noir
/// fn find_even(values: [u32; 3]) -> Option<u32> {
///     let mut result = Option::none();
///     for value in values {
///         if result.is_none() & (value % 2 == 0) {
///             result = Option::some(value);
///         }
///     }
///     result
/// }
///
/// fn main() {
///     assert_eq(find_even([1, 4, 6]), Option::some(4));
///     assert(find_even([1, 3, 5]).is_none());
/// }
/// ```
pub struct Option<T> {
    _is_some: bool,
    _value: T,
}

impl<T> Option<T> {
    /// Constructs a None value
    ///
    /// Example:
    ///
    /// ```noir
    /// let x: Option<u32> = Option::none();
    /// assert(x.is_none());
    /// ```
    pub fn none() -> Self {
        Self { _is_some: false, _value: crate::mem::zeroed() }
    }

    /// Constructs a Some wrapper around the given value
    ///
    /// Example:
    ///
    /// ```noir
    /// let x = Option::some(5);
    /// assert(x.is_some());
    /// assert_eq(x.unwrap(), 5);
    /// ```
    pub fn some(_value: T) -> Self {
        Self { _is_some: true, _value }
    }

    /// True if this Option is None
    ///
    /// Example:
    ///
    /// ```noir
    /// assert(Option::<u32>::none().is_none());
    /// assert(!Option::some(5).is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        !self._is_some
    }

    /// True if this Option is Some
    ///
    /// Example:
    ///
    /// ```noir
    /// assert(Option::some(5).is_some());
    /// assert(!Option::<u32>::none().is_some());
    /// ```
    pub fn is_some(&self) -> bool {
        self._is_some
    }

    /// Asserts `self.is_some()` and returns the wrapped value.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(5).unwrap(), 5);
    /// ```
    ///
    /// # Failures
    ///
    /// Fails if `self` is `None`.
    ///
    /// ```noir should_fail
    /// let x: Option<u32> = Option::none();
    /// // Fails: `x` is `None`.
    /// let _ = x.unwrap();
    /// ```
    pub fn unwrap(self) -> T {
        assert(self._is_some);
        self._value
    }

    /// Returns the inner value without asserting `self.is_some()`
    /// Note that if `self` is `None`, there is no guarantee what value will be returned,
    /// only that it will be of type `T`.
    ///
    /// Use it only where `self.is_some()` has already been established, for example to avoid
    /// repeating an assertion.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn value_or_zero(x: Option<u32>) -> u32 {
    ///     if x.is_some() {
    ///         x.unwrap_unchecked()
    ///     } else {
    ///         0
    ///     }
    /// }
    ///
    /// fn main() {
    ///     assert_eq(value_or_zero(Option::some(5)), 5);
    ///     assert_eq(value_or_zero(Option::none()), 0);
    /// }
    /// ```
    pub fn unwrap_unchecked(self) -> T {
        self._value
    }

    /// Returns the wrapped value if `self.is_some()`. Otherwise, returns the given default value.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(5).unwrap_or(0), 5);
    /// assert_eq(Option::none().unwrap_or(0), 0);
    /// ```
    pub fn unwrap_or(self, default: T) -> T {
        if self._is_some {
            self._value
        } else {
            default
        }
    }

    /// Returns the wrapped value if `self.is_some()`. Otherwise, calls the given function to return
    /// a default value.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(5).unwrap_or_else(|| 0), 5);
    /// assert_eq(Option::none().unwrap_or_else(|| 7), 7);
    /// ```
    pub fn unwrap_or_else<Env>(self, default: fn[Env]() -> T) -> T {
        if self._is_some {
            self._value
        } else {
            default()
        }
    }

    /// Asserts `self.is_some()` with a provided custom message and returns the contained `Some` value
    ///
    /// Example:
    ///
    /// ```noir
    /// let x = Option::some(5);
    /// assert_eq(x.expect(f"x should be set"), 5);
    /// ```
    ///
    /// # Failures
    ///
    /// Fails with `message` if `self` is `None`.
    ///
    /// ```noir should_fail
    /// let x: Option<u32> = Option::none();
    /// // Fails with the message "x should be set".
    /// let _ = x.expect(f"x should be set");
    /// ```
    pub fn expect<let N: u32, MessageTypes>(self, message: fmtstr<N, MessageTypes>) -> T {
        assert(self.is_some(), message);
        self._value
    }

    /// If self is `Some(x)`, this returns `Some(f(x))`. Otherwise, this returns `None`.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(5).map(|x| x * 2), Option::some(10));
    /// assert(Option::<u32>::none().map(|x| x * 2).is_none());
    /// ```
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> Option<U> {
        if self._is_some {
            Option::some(f(self._value))
        } else {
            Option::none()
        }
    }

    /// If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns the given default value.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(5).map_or(0, |x| x * 2), 10);
    /// assert_eq(Option::none().map_or(0, |x: u32| x * 2), 0);
    /// ```
    pub fn map_or<U, Env>(self, default: U, f: fn[Env](T) -> U) -> U {
        if self._is_some {
            f(self._value)
        } else {
            default
        }
    }

    /// If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns `default()`.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(5).map_or_else(|| 0, |x| x * 2), 10);
    /// assert_eq(Option::none().map_or_else(|| 0, |x: u32| x * 2), 0);
    /// ```
    pub fn map_or_else<U, Env1, Env2>(self, default: fn[Env1]() -> U, f: fn[Env2](T) -> U) -> U {
        if self._is_some {
            f(self._value)
        } else {
            default()
        }
    }

    /// Returns None if self is None. Otherwise, this returns `other`.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(1).and(Option::some(2)), Option::some(2));
    /// assert(Option::<u32>::none().and(Option::some(2)).is_none());
    /// ```
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
    ///
    /// Example:
    ///
    /// ```noir
    /// let halve = |x: u32| if x % 2 == 0 { Option::some(x / 2) } else { Option::none() };
    /// assert_eq(Option::some(8).and_then(halve), Option::some(4));
    /// assert(Option::some(3).and_then(halve).is_none());
    /// ```
    pub fn and_then<U, Env>(self, f: fn[Env](T) -> Option<U>) -> Option<U> {
        if self._is_some {
            f(self._value)
        } else {
            Option::none()
        }
    }

    /// If self is Some, return self. Otherwise, return `other`.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(1).or(Option::some(2)), Option::some(1));
    /// assert_eq(Option::none().or(Option::some(2)), Option::some(2));
    /// ```
    pub fn or(self, other: Self) -> Self {
        if self._is_some {
            self
        } else {
            other
        }
    }

    /// If self is Some, return self. Otherwise, return `default()`.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(1).or_else(|| Option::some(2)), Option::some(1));
    /// assert_eq(Option::none().or_else(|| Option::some(2)), Option::some(2));
    /// ```
    pub fn or_else<Env>(self, default: fn[Env]() -> Self) -> Self {
        if self._is_some {
            self
        } else {
            default()
        }
    }

    /// If only one of the two Options is Some, return that option.
    /// Otherwise, if both options are Some or both are None, None is returned.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(1).xor(Option::none()), Option::some(1));
    /// assert(Option::some(1).xor(Option::some(2)).is_none());
    /// assert(Option::<u32>::none().xor(Option::none()).is_none());
    /// ```
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
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::some(4).filter(|x| x % 2 == 0), Option::some(4));
    /// assert(Option::some(3).filter(|x| x % 2 == 0).is_none());
    /// ```
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
    ///
    /// Example:
    ///
    /// ```noir
    /// assert_eq(Option::flatten(Option::some(Option::some(5))), Option::some(5));
    /// assert(Option::flatten(Option::some(Option::<u32>::none())).is_none());
    /// ```
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

mod tests {
    use crate::cmp::Ord;
    use crate::cmp::Ordering;
    use crate::default::Default as _;
    use super::Option;

    #[test]
    fn some_and_none() {
        assert(Option::<u8>::none().is_none());
        assert(!Option::<u8>::none().is_some());
        assert(Option::some(1).is_some());
        assert(!Option::some(1).is_none());
    }

    #[test]
    fn unwrap_succeeds() {
        assert_eq(Option::some(1).unwrap(), 1);
    }

    #[test(should_fail)]
    fn unwrap_fails() {
        let _ = Option::<u8>::none().unwrap();
    }

    #[test]
    fn unwrap_or() {
        assert_eq(Option::some(1).unwrap_or(2), 1);
        assert_eq(Option::none().unwrap_or(2), 2);
    }

    #[test]
    fn unwrap_or_else() {
        assert_eq(Option::some(1).unwrap_or_else(|| 2), 1);
        assert_eq(Option::none().unwrap_or_else(|| 2), 2);
    }

    #[test]
    fn expect_succeeds() {
        assert_eq(Option::some(1).expect(f"Should be there"), 1);
    }

    #[test(should_fail_with = "Should be there")]
    fn expect_fails() {
        let _ = Option::<u8>::none().expect(f"Should be there");
    }

    #[test]
    fn map() {
        assert(Option::<u8>::none().map(|x| x + 1).is_none());
        assert_eq(Option::some(1).map(|x| x + 1), Option::some(2));
    }

    #[test]
    fn map_or() {
        assert_eq(Option::<u8>::none().map_or(0, |x| x + 1), 0);
        assert_eq(Option::some(1).map_or(0, |x| x + 1), 2);
    }

    #[test]
    fn map_or_else() {
        assert_eq(Option::<u8>::none().map_or_else(|| 0, |x| x + 1), 0);
        assert_eq(Option::some(1).map_or_else(|| 0, |x| x + 1), 2);
    }

    #[test]
    fn and() {
        assert_eq(Option::<u8>::none().and(Option::none()), Option::none());
        assert_eq(Option::<u8>::none().and(Option::some(1)), Option::none());
        assert_eq(Option::some(1).and(Option::some(2)), Option::some(2));
        assert_eq(Option::some(1).and(Option::none()), Option::none());
    }

    #[test]
    fn and_then() {
        assert_eq(Option::<u8>::none().and_then(|_| Option::<u8>::none()), Option::none());
        assert_eq(Option::<u8>::none().and_then(|_| Option::some(1)), Option::none());
        assert_eq(Option::some(1).and_then(|x| Option::some(x + 1)), Option::some(2));
        assert_eq(Option::some(1).and_then(|_| Option::<u8>::none()), Option::none());
    }

    #[test]
    fn or() {
        assert_eq(Option::<u8>::none().or(Option::none()), Option::none());
        assert_eq(Option::<u8>::none().or(Option::some(1)), Option::some(1));
        assert_eq(Option::some(1).or(Option::some(2)), Option::some(1));
        assert_eq(Option::some(1).or(Option::none()), Option::some(1));
    }

    #[test]
    fn or_else() {
        assert_eq(Option::<u8>::none().or_else(|| Option::none()), Option::none());
        assert_eq(Option::<u8>::none().or_else(|| Option::some(1)), Option::some(1));
        assert_eq(Option::some(1).or_else(|| Option::some(2)), Option::some(1));
        assert_eq(Option::some(1).or_else(|| Option::none()), Option::some(1));
    }

    #[test]
    fn xor() {
        assert_eq(Option::<u8>::none().xor(Option::none()), Option::none());
        assert_eq(Option::<u8>::none().xor(Option::some(1)), Option::some(1));
        assert_eq(Option::some(1).xor(Option::some(2)), Option::none());
        assert_eq(Option::some(1).xor(Option::none()), Option::some(1));
    }

    #[test]
    fn filter() {
        assert_eq(Option::<u8>::none().filter(|_| true), Option::none());
        assert_eq(Option::some(1).filter(|x| x == 1), Option::some(1));
        assert_eq(Option::some(1).filter(|x| x == 2), Option::none());
        assert_eq(Option::some(1).filter(|x| x == 2), Option::none());
    }

    #[test]
    fn flatten() {
        assert_eq(Option::<Option<u8>>::none().flatten(), Option::none());
        assert_eq(Option::some(Option::<u8>::none()).flatten(), Option::none());
        assert_eq(Option::some(Option::some(1)).flatten(), Option::some(1));
    }

    #[test]
    fn default() {
        assert_eq(Option::<u8>::default(), Option::none());
    }

    #[test]
    fn eq() {
        assert(Option::<u8>::none() == Option::none());
        assert(Option::<u8>::some(1) != Option::none());
        assert(Option::<u8>::none() != Option::some(1));
        assert(Option::<u8>::some(1) == Option::some(1));
        assert(Option::<u8>::some(1) != Option::some(2));
    }

    #[test]
    fn cmp() {
        let none = Option::<u8>::none();
        let one = Option::<u8>::some(1);
        let two = Option::<u8>::some(2);
        assert_eq(none.cmp(none), Ordering::equal());
        assert_eq(none.cmp(one), Ordering::less());
        assert_eq(one.cmp(none), Ordering::greater());
        assert_eq(one.cmp(one), Ordering::equal());
        assert_eq(one.cmp(two), Ordering::less());
        assert_eq(two.cmp(one), Ordering::greater());
    }
}
