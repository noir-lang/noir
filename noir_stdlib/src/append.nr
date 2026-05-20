/// Appends two values together, returning the result.
///
/// An alternate name for this trait is `Monoid` if that is familiar.
/// If not, it can be ignored.
///
/// It is expected that for any implementation:
/// - `T::empty().append(x) == x`
/// - `x.append(T::empty()) == x`
// docs:start:append-trait
pub trait Append {
    fn empty() -> Self;
    fn append(self, other: Self) -> Self;
}
// docs:end:append-trait

impl<T> Append for [T] {
    fn empty() -> Self {
        [].as_vector()
    }

    fn append(self, other: Self) -> Self {
        // Vectors have an existing append function which this will resolve to.
        self.append(other)
    }
}

impl Append for Quoted {
    comptime fn empty() -> Self {
        quote {}
    }

    comptime fn append(self, other: Self) -> Self {
        quote { $self $other }
    }
}

mod tests {
    use super::Append;

    #[test]
    fn slice_empty() {
        assert_eq(<[Field]>::empty(), @[]);
    }

    #[test]
    fn slice_append() {
        assert_eq(Append::append(@[1, 2], [3, 4, 5]), @[1, 2, 3, 4, 5]);
    }

    #[test]
    fn quoted_append() {
        comptime {
            assert_eq(Append::append(quote { hello }, quote { world }), quote { hello world })
        }
    }

    #[test]
    fn quoted_empty() {
        comptime {
            assert_eq(<Quoted>::empty(), quote {});
        }
    }
}
