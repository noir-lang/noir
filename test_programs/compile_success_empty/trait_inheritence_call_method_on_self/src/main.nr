pub trait Empty: Eq {
    fn empty() -> Self;

    fn is_empty(self) -> bool {
        self.eq(Self::empty())
    }
}

fn main() {}
