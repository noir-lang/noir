use crate::cmp::{Eq, Ord};
use crate::convert::From;
use crate::runtime::is_unconstrained;

mod check_shuffle;
mod quicksort;

impl<T, let N: u32> [T; N] {
    /// Returns the length of the slice.
    #[builtin(array_len)]
    pub fn len(self) -> u32 {}

    #[builtin(as_slice)]
    pub fn as_slice(self) -> [T] {}

    // Apply a function to each element of an array, returning a new array
    // containing the mapped elements.
    pub fn map<U, Env>(self, f: fn[Env](T) -> U) -> [U; N] {
        let first_elem = f(self[0]);
        let mut ret = [first_elem; N];

        for i in 1..self.len() {
            ret[i] = f(self[i]);
        }

        ret
    }

    // Apply a function to each element of the array and an accumulator value,
    // returning the final accumulated value. This function is also sometimes
    // called `foldl`, `fold_left`, `reduce`, or `inject`.
    pub fn fold<U, Env>(self, mut accumulator: U, f: fn[Env](U, T) -> U) -> U {
        for elem in self {
            accumulator = f(accumulator, elem);
        }
        accumulator
    }

    // Apply a function to each element of the array and an accumulator value,
    // returning the final accumulated value. Unlike fold, reduce uses the first
    // element of the given array as its starting accumulator value.
    pub fn reduce<Env>(self, f: fn[Env](T, T) -> T) -> T {
        let mut accumulator = self[0];
        for i in 1..self.len() {
            accumulator = f(accumulator, self[i]);
        }
        accumulator
    }

    // Returns true if all elements in the array satisfy the predicate
    pub fn all<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = true;
        for elem in self {
            ret &= predicate(elem);
        }
        ret
    }

    // Returns true if any element in the array satisfies the predicate
    pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
        let mut ret = false;
        for elem in self {
            ret |= predicate(elem);
        }
        ret
    }
}

impl<T, let N: u32> [T; N] where T: Ord + Eq {
    pub fn sort(self) -> Self {
        self.sort_via(|a: T, b: T| a <= b)
    }
}

impl<T, let N: u32> [T; N] where T: Eq {

    /// Sorts the array using a custom predicate function `ordering`.
    ///
    /// The `ordering` function must be designed to return `true` for equal valued inputs
    /// If this is not done, `sort_via` will fail to sort inputs with duplicated elements.
    pub fn sort_via<Env>(self, ordering: fn[Env](T, T) -> bool) -> Self {
        unsafe {
            // Safety: `sorted` array is checked to be:
            //   a. a permutation of `input`'s elements
            //   b. satisfying the predicate `ordering`
            let sorted = quicksort::quicksort(self, ordering);

            if !is_unconstrained() {
                for i in 0..N - 1 {
                    assert(
                        ordering(sorted[i], sorted[i + 1]), "Array has not been sorted correctly according to `ordering`."
                    );
                }
                check_shuffle::check_shuffle(self, sorted);
            }
            sorted
        }
    }
}

impl<let N: u32> [u8; N] {
    /// Convert a sequence of bytes as-is into a string.
    /// This function performs no UTF-8 validation or similar.
    #[builtin(array_as_str_unchecked)]
    pub fn as_str_unchecked(self) -> str<N> {}
}

impl<let N: u32> From<str<N>> for [u8; N] {
    fn from(s: str<N>) -> Self {
        s.as_bytes()
    }
}
