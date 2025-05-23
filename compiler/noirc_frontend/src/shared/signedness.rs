#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Signedness {
    Unsigned,
    Signed,
}

impl Signedness {
    pub fn is_signed(&self) -> bool {
        match self {
            Signedness::Unsigned => false,
            Signedness::Signed => true,
        }
    }
}
