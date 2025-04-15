trait Default2 {
    fn default2() -> Self;
}

trait Eq2 {
    fn eq2(self, other: Self) -> bool;
}

trait IsDefault {
    fn is_default(self) -> bool;
}

impl<T> IsDefault for T
where
    T: Default2 + Eq2,
{
    fn is_default(self) -> bool {
        self.eq2(T::default2())
    }
}

struct Foo {
    a: u64,
}

impl Eq2 for Foo {
    fn eq2(self, other: Foo) -> bool {
        self.a == other.a
    }
}

impl Default2 for u64 {
    fn default2() -> Self {
        0
    }
}

impl Default2 for Foo {
    fn default2() -> Self {
        Foo { a: Default2::default2() }
    }
}

fn main(a: Foo) -> pub bool {
    a.is_default()
}
