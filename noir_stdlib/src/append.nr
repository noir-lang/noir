// Appends two values together, returning the result.
//
// An alternate name for this trait is `Monoid` if that is familiar.
// If not, it can be ignored.
//
// It is expected that for any implementation:
// - `T::empty().append(x) == x`
// - `x.append(T::empty()) == x`
trait Append {
    fn empty() -> Self;
    fn append(self, other: Self) -> Self;
}

impl<T> Append for [T] {
    fn empty() -> Self {
        &[]
    }

    fn append(self, other: Self) -> Self {
        // Slices have an existing append function which this will resolve to.
        self.append(other)
    }
}

impl Append for Quoted {
    fn empty() -> Self {
        quote {}
    }

    fn append(self, other: Self) -> Self {
        quote { $self $other }
    }
}
