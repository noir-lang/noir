use crate::append::Append;

impl CtString {
    /// Returns an empty `CtString`.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::meta::ctstring::AsCtString;
    ///
    /// comptime {
    ///     let empty = CtString::new();
    ///     assert_eq(empty.append_str("abc"), "abc".as_ctstring());
    /// }
    /// ```
    // docs:start:new
    pub comptime fn new() -> Self {
        // docs:end:new
        "".as_ctstring()
    }

    /// Returns a new `CtString` with `s` appended to the end.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::meta::ctstring::AsCtString;
    ///
    /// comptime {
    ///     let greeting = "hello".as_ctstring().append_str(" world");
    ///     assert_eq(greeting, "hello world".as_ctstring());
    /// }
    /// ```
    // TODO(https://github.com/noir-lang/noir/issues/6980): Bug: using &mut self
    // as the object results in this method not being found
    // docs:start:append_str
    pub comptime fn append_str<let N: u32>(self, s: str<N>) -> Self {
        // docs:end:append_str
        self.append(s.as_ctstring())
    }

    /// Returns a new `CtString` with the formatted string `s` appended to the end.
    ///
    /// Values interpolated into `s` are formatted the same way as in `println`, so a `Field` is
    /// written in hexadecimal.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::meta::ctstring::AsCtString;
    ///
    /// comptime {
    ///     let count = 3_u32;
    ///     let message = "count: ".as_ctstring().append_fmtstr(f"{count}");
    ///     assert_eq(message, "count: 3".as_ctstring());
    /// }
    /// ```
    // docs:start:append_fmtstr
    pub comptime fn append_fmtstr<let N: u32, T>(self, s: fmtstr<N, T>) -> Self {
        // docs:end:append_fmtstr
        self.append(s.as_ctstring())
    }

    /// Returns this string as a quoted string literal.
    ///
    /// A `CtString` can't be converted to a `str<N>` directly, because its length isn't known in its
    /// type. Insert the literal with a macro call instead: `foo.as_quoted_str!()`.
    ///
    /// Example:
    ///
    /// ```noir
    /// use std::meta::ctstring::AsCtString;
    ///
    /// comptime {
    ///     let name = "foo".as_ctstring().append_str("bar");
    ///     let name_str: str<6> = name.as_quoted_str!();
    ///     assert_eq(name_str, "foobar");
    /// }
    /// ```
    // docs:start:as_quoted_str
    pub comptime fn as_quoted_str(self) -> Quoted {
        // docs:end:as_quoted_str
        quote { $self }
    }
}

impl Append for CtString {
    comptime fn empty() -> Self {
        "".as_ctstring()
    }

    #[builtin(ctstring_append)]
    comptime fn append(self, other: Self) -> Self {}
}

/// Converts a string into a [`CtString`] at compile time.
///
/// It is implemented for `str<N>` and `fmtstr<N, T>`.
///
/// Example:
///
/// ```noir
/// use std::meta::ctstring::AsCtString;
///
/// comptime {
///     let x = 7_u32;
///     assert_eq(f"x = {x}".as_ctstring(), "x = 7".as_ctstring());
/// }
/// ```
// docs:start:as-ctstring
pub trait AsCtString {
    /// Returns this string as a `CtString`.
    comptime fn as_ctstring(self) -> CtString;
}
// docs:end:as-ctstring

impl<let N: u32> AsCtString for str<N> {
    #[builtin(str_as_ctstring)]
    comptime fn as_ctstring(self) -> CtString {}
}

impl<let N: u32, T> AsCtString for fmtstr<N, T> {
    #[builtin(fmtstr_as_ctstring)]
    comptime fn as_ctstring(self) -> CtString {}
}

impl crate::cmp::Eq for CtString {
    #[builtin(ctstring_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

impl crate::hash::Hash for CtString {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        state.write(ctstring_hash(self));
    }
}

#[builtin(ctstring_hash)]
comptime fn ctstring_hash(_string: CtString) -> Field {}

mod test {
    use super::Append as _;
    use super::AsCtString;

    #[test]
    fn as_quoted_str_example() {
        comptime {
            // docs:start:as_quoted_str_example
            let my_ctstring = "foo bar".as_ctstring();
            let my_str: str<7> = my_ctstring.as_quoted_str!();

            assert_eq(crate::meta::type_of(my_str), quote { str<7> }.as_type());
            // docs:end:as_quoted_str_example
        }
    }

    #[test]
    fn empty() {
        comptime {
            assert_eq(CtString::empty(), CtString::new());
        }
    }

    #[test]
    fn append() {
        comptime {
            let hello = "hello ".as_ctstring();
            let world = "world".as_ctstring();
            assert_eq(hello.append(world), "hello world".as_ctstring());
        }
    }

}
