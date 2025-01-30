use crate::cmp::{Eq, Ord};
use crate::convert::From;
use crate::runtime::is_unconstrained;

mod check_shuffle;
mod quicksort;

impl<T, let N: u32> [T; N] {
    /// Returns the length of this array.
    ///
    /// ```noir
    /// fn len(self) -> Field
    /// ```
    ///
    /// example
    ///
    /// ```noir
    /// fn main() {
    ///     let array = [42, 42];
    ///     assert(array.len() == 2);
    /// }
    /// ```
    #[builtin(array_len)]
    pub fn len(self) -> u32 {}

    /// Returns this array as a slice.
    ///
    /// ```noir
    /// let array = [1, 2];
    /// let slice = array.as_slice();
    /// assert_eq(slice, &[1, 2]);
    /// ```
    #[builtin(as_slice)]
    pub fn as_slice(self) -> [T] {}

    /// Applies a function to each element of this array, returning a new array containing the mapped elements.
    ///
    /// Example:
    ///
    /// ```rust
    /// let a = [1, 2, 3];
    /// let b = a.map(|a| a * 2);
    /// assert_eq(b, [2, 4, 6]);
    /// ```
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> [U; N] {
        let uninitialized = crate::mem::zeroed();
        let mut ret = [uninitialized; N];

        for i in 0..self.len() {
            ret[i] = f(self[i]);
        }

        ret
    }

    /// Applies a function to each element of the array, returning the final accumulated value. The first
    /// parameter is the initial value.
    ///
    /// This is a left fold, so the given function will be applied to the accumulator and first element of
    /// the array, then the second, and so on. For a given call the expected result would be equivalent to:
    ///
    /// ```rust
    /// let a1 = [1];
    /// let a2 = [1, 2];
    /// let a3 = [1, 2, 3];
    ///
    /// let f = |a, b| a - b;
    /// a1.fold(10, f); //=> f(10, 1)
    /// a2.fold(10, f); //=> f(f(10, 1), 2)
    /// a3.fold(10, f); //=> f(f(f(10, 1), 2), 3)
    ///
    /// assert_eq(a3.fold(10, f), 10 - 1 - 2 - 3);
    /// ```
    pub fn fold<U, Env>(self, mut accumulator: U, f: fn[Env](U, T) -> U) -> U {
        for elem in self {
            accumulator = f(accumulator, elem);
        }
        accumulator
    }

    /// Same as fold, but uses the first element as the starting element.
    ///
    /// Requires the input array to be non-empty.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn main() {
    ///     let arr = [1, 2, 3, 4];
    ///     let reduced = arr.reduce(|a, b| a + b);
    ///     assert(reduced == 10);
    /// }
    /// ```
    pub fn reduce<Env>(self, f: fn[Env](T, T) -> T) -> T {
        let mut accumulator = self[0];
        for i in 1..self.len() {
            accumulator = f(accumulator, self[i]);
        }
        accumulator
    }

    /// Returns true if all the elements in this array satisfy the given predicate.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn main() {
    ///     let arr = [2, 2, 2, 2, 2];
    ///     let all = arr.all(|a| a == 2);
    ///     assert(all);
    /// }
    /// ```
    pub fn all<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = true;
        for elem in self {
            ret &= predicate(elem);
        }
        ret
    }

    /// Returns true if any of the elements in this array satisfy the given predicate.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn main() {
    ///     let arr = [2, 2, 2, 2, 5];
    ///     let any = arr.any(|a| a == 5);
    ///     assert(any);
    /// }
    /// ```
    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        for elem in self {
            ret |= predicate(elem);
        }
        ret
    }
}

impl<T, let N: u32> [T; N]
where
    T: Ord + Eq,
{
    /// Returns a new sorted array. The original array remains untouched. Notice that this function will
    /// only work for arrays of fields or integers, not for any arbitrary type. This is because the sorting
    /// logic it uses internally is optimized specifically for these values. If you need a sort function to
    /// sort any type, you should use the `sort_via` function.
    ///
    /// Example:
    ///
    /// ```rust
    /// fn main() {
    ///     let arr = [42, 32];
    ///     let sorted = arr.sort();
    ///     assert(sorted == [32, 42]);
    /// }
    /// ```
    pub fn sort(self) -> Self {
        self.sort_via(|a, b| a <= b)
    }
}

impl<T, let N: u32> [T; N]
where
    T: Eq,
{
    /// Returns a new sorted array by sorting it with a custom comparison function.
    /// The original array remains untouched.
    /// The ordering function must return true if the first argument should be sorted to be before the second argument or is equal to the second argument.
    ///
    /// Using this method with an operator like `<` that does not return `true` for equal values will result in an assertion failure for arrays with equal elements.
    ///
    /// Example:
    ///
    /// ```rust
    /// fn main() {
    ///     let arr = [42, 32]
    ///     let sorted_ascending = arr.sort_via(|a, b| a <= b);
    ///     assert(sorted_ascending == [32, 42]); // verifies
    ///
    ///     let sorted_descending = arr.sort_via(|a, b| a >= b);
    ///     assert(sorted_descending == [32, 42]); // does not verify
    /// }
    /// ```
    pub fn sort_via<Env>(self, ordering: fn[Env](T, T) -> bool) -> Self {
        /// Safety: `sorted` array is checked to be:
        /// a. a permutation of `input`'s elements
        /// b. satisfying the predicate `ordering`
        unsafe {
            let sorted = quicksort::quicksort(self, ordering);

            if !is_unconstrained() {
                for i in 0..N - 1 {
                    assert(
                        ordering(sorted[i], sorted[i + 1]),
                        "Array has not been sorted correctly according to `ordering`.",
                    );
                }
                check_shuffle::check_shuffle(self, sorted);
            }
            sorted
        }
    }
}

impl<let N: u32> [u8; N] {
    /// Converts a byte array of type `[u8; N]` to a string. Note that this performs no UTF-8 validation -
    /// the given array is interpreted as-is as a string.
    ///
    /// Example:
    ///
    /// ```rust
    /// fn main() {
    ///     let hi = [104, 105].as_str_unchecked();
    ///     assert_eq(hi, "hi");
    /// }
    /// ```
    #[builtin(array_as_str_unchecked)]
    pub fn as_str_unchecked(self) -> str<N> {}
}

impl<let N: u32> From<str<N>> for [u8; N] {
    /// Returns an array of the string bytes.
    fn from(s: str<N>) -> Self {
        s.as_bytes()
    }
}

mod test {
    #[test]
    fn map_empty() {
        assert_eq([].map(|x| x + 1), []);
    }
}
