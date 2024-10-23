trait Bar {
    fn ok(self) -> Self;

    fn ref_ok(self) -> Self {
        self.ok()
    }
}

impl<T> Bar for (T, T)
where
    T: Bar,
{
    fn ok(self) -> Self {
        self
    }
}

fn main() {}
