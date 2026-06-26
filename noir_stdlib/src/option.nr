use crate::cmp::{Eq, Ord, Ordering};
use crate::default::Default;
use crate::hash::{Hash, Hasher};
use crate::validate::Validate;

/// Represents a value of type T or its absence.
/// Use `Option::some(value)` to construct a value or `Option::none()` to record the absence of one.
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
    pub fn is_none(&self) -> bool {
        !self._is_some
    }

    /// True if this Option is Some
    pub fn is_some(&self) -> bool {
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

impl<T> Validate for Option<T>
where
    T: Validate,
{
    fn validate(&self) {
        if self._is_some {
            self._value.validate();
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
