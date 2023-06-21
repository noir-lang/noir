struct Option<T> {
    _is_some: bool,
    value: T,
}

impl<T> Option<T> {
    fn none() -> Self {
        Self { _is_some: false, value: crate::unsafe::zeroed() }
    }

    fn some(value: T) -> Self {
        Self { _is_some: true, value }
    }

    fn is_none(self) -> bool {
        !self._is_some
    }

    fn is_some(self) -> bool {
        self._is_some
    }

    fn unwrap(self) -> T {
        assert(self._is_some);
        self.value
    }

    fn unwrap_or(self, default: T) -> T {
        if self._is_some {
            self.value
        } else {
            default
        }
    }

    fn unwrap_or_else(self, default: fn() -> T) -> T {
        if self._is_some {
            self.value
        } else {
            default()
        }
    }

    fn map<U>(self, f: fn(T) -> U) -> Option<U> {
        if self._is_some {
            Option::some(f(self.value))
        } else {
            Option::none()
        }
    }

    fn map_or<U>(self, default: U, f: fn(T) -> U) -> U {
        if self._is_some {
            f(self.value)
        } else {
            default
        }
    }

    fn map_or_else<U>(self, default: fn() -> U, f: fn(T) -> U) -> U {
        if self._is_some {
            f(self.value)
        } else {
            default()
        }
    }

    fn and(self, other: Self) -> Self {
        if self.is_none() {
            Option::none()
        } else {
            other
        }
    }

    fn and_then<U>(self, f: fn(T) -> Option<U>) -> Option<U> {
        if self._is_some {
            f(self.value)
        } else {
            Option::none()
        }
    }

    fn or(self, other: Self) -> Self {
        if self._is_some {
            self
        } else {
            other
        }
    }

    fn or_else<U>(self, default: fn() -> Option<T>) -> Option<T> {
        if self._is_some {
            self
        } else {
            default()
        }
    }

    fn xor(self, other: Self) -> Self {
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
}
