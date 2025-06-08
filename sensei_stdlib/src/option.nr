use crate::cmp::{Eq, Ord, Ordering};
use crate::default::Default;
use crate::hash::{Hash, Hasher};
use crate::panic::panic;
use crate::hint::unchecked_unreachable;

pub enum Option<T> {
    Some(T),
    None,
}

impl<T> Option<T> {
    /// True if this Option is None
    pub fn is_none(self) -> bool {
        match self {
            Self::Some(_) => false,
            Self::None => true,
        }
    }

    /// True if this Option is Some
    pub fn is_some(self) -> bool {
        match self {
            Self::Some(_) => true,
            Self::None => false,
        }
    }

    /// Asserts `self.is_some()` and returns the wrapped value.
    pub fn unwrap(self) -> T {
        match self {
            Self::Some(a) => a,
            Self::None => panic(f"unwrapping on a None value"),
        }
    }

    /// Returns the inner value without asserting `self.is_some()`
    /// Note that if `self` is `None`, there is no guarantee what value will be returned,
    /// only that it will be of type `T`.
    pub fn unwrap_unchecked(self) -> T {
        match self {
            Self::Some(value) => value,
            Self::None => unchecked_unreachable(),
        }
    }

    /// Returns the wrapped value if `self.is_some()`. Otherwise, returns the given default value.
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Some(value) => value,
            Self::None => default,
        }
    }

    /// Returns the wrapped value if `self.is_some()`. Otherwise, calls the given function to return
    /// a default value.
    pub fn unwrap_or_else<Env>(self, default: fn[Env]() -> T) -> T {
        match self {
            Self::Some(value) => value,
            Self::None => default(),
        }
    }

    /// Asserts `self.is_some()` with a provided custom message and returns the contained `Some` value
    pub fn expect<let N: u32, MessageTypes>(self, message: fmtstr<N, MessageTypes>) -> T {
        match self {
            Self::Some(value) => value,
            Self::None => panic(message),
        }
    }

    /// If self is `Some(x)`, this returns `Some(f(x))`. Otherwise, this returns `None`.
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> Option<U> {
        match self {
            Self::Some(value) => Option::Some(f(value)),
            Self::None => Self::None,
        }
    }

    /// If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns the given default value.
    pub fn map_or<U, Env>(self, default: U, f: fn[Env](T) -> U) -> U {
        match self {
            Self::Some(value) => f(value),
            Self::None => default,
        }
    }

    /// If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns `default()`.
    pub fn map_or_else<U, Env1, Env2>(self, default: fn[Env1]() -> U, f: fn[Env2](T) -> U) -> U {
        match self {
            Self::Some(value) => f(value),
            Self::None => default(),
        }
    }

    /// Returns None if self is None. Otherwise, this returns `other`.
    pub fn and(self, other: Self) -> Self {
        match self {
            Self::Some(_) => other,
            Self::None => Self::None,
        }
    }

    /// If self is None, this returns None. Otherwise, this calls the given function
    /// with the Some value contained within self, and returns the result of that call.
    ///
    /// In some languages this function is called `flat_map` or `bind`.
    pub fn and_then<U, Env>(self, f: fn[Env](T) -> Option<U>) -> Option<U> {
        match self {
            Self::Some(value) => f(value),
            Self::None => Self::None,
        }
    }

    /// If self is Some, return self. Otherwise, return `other`.
    pub fn or(self, other: Self) -> Self {
        match self {
            Self::Some(_) => self,
            Self::None => other,
        }
    }

    /// If self is Some, return self. Otherwise, return `default()`.
    pub fn or_else<Env>(self, default: fn[Env]() -> Self) -> Self {
        match self {
            Self::Some(_) => self,
            Self::None => default(),
        }
    }

    // If only one of the two Options is Some, return that option.
    // Otherwise, if both options are Some or both are None, None is returned.
    pub fn xor(self, other: Self) -> Self {
        match self {
            Self::Some(_) => match other {
                Self::Some(_) => Self::None,
                Self::None => self,
            },
            Self::None => other,
        }
    }

    /// Returns `Some(x)` if self is `Some(x)` and `predicate(x)` is true.
    /// Otherwise, this returns `None`
    pub fn filter<Env>(self, predicate: fn[Env](T) -> bool) -> Self {
        match self {
            Self::Some(value) => if predicate(value) {
                self
            } else {
                Self::None
            },
            Self::None => Self::None,
        }
    }

    /// Flattens an Option<Option<T>> into a Option<T>.
    /// This returns None if the outer Option is None. Otherwise, this returns the inner Option.
    pub fn flatten(option: Option<Option<T>>) -> Option<T> {
         match option {
            Self::Some(value) => value,
            Self::None => Self::None,
        }
    }

}


impl<T> Default for Option<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T> Eq for Option<T>
where
    T: Eq,
{
    fn eq(self, other: Self) -> bool {
         match self {
            Self::Some(value) => match other {
                Self::Some(other_value) => value == other_value,
                Self::None => false,
            },
            Self::None => other.is_none(),
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
        self.is_some().hash(state);
        let _ = self.map(|value| value.hash(state));
    }
}

// For this impl we're declaring Option::none < Option::some
impl<T> Ord for Option<T>
where
    T: Ord,
{
    fn cmp(self, other: Self) -> Ordering {
        match self {
            Self::Some(value) => match other {
                Self::Some(other_value) => value.cmp(other_value),
                Self::None => Ordering::Greater,
            },
            Self::None => if other.is_some() {
                Ordering::Less
            } else {
                Ordering::Equal
            },
        }
    }
}
