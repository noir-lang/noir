use crate::append::Append;

impl CtString {
    // docs:start:new
    pub comptime fn new() -> Self {
        // docs:end:new
        "".as_ctstring()
    }

    // TODO(https://github.com/noir-lang/noir/issues/6980): Bug: using &mut self
    // as the object results in this method not being found
    // docs:start:append_str
    pub comptime fn append_str<let N: u32>(self, s: str<N>) -> Self {
        // docs:end:append_str
        f"{self}{s}".as_ctstring()
    }

    // docs:start:append_fmtstr
    pub comptime fn append_fmtstr<let N: u32, T>(self, s: fmtstr<N, T>) -> Self {
        // docs:end:append_fmtstr
        f"{self}{s}".as_ctstring()
    }

    /// CtString cannot directly return a str since the size would not be known.
    /// To get around this, we return a quoted str and the underlying str can
    /// be accessed using macro insertion `foo.as_quoted_str!()`.
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

    comptime fn append(self, other: Self) -> Self {
        f"{self}{other}".as_ctstring()
    }
}

// docs:start:as-ctstring
pub trait AsCtString {
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
