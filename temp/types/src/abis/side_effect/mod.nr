pub mod counted;
pub mod scoped;

pub use counted::Counted;

use crate::address::AztecAddress;

pub trait Ordered {
    fn counter(self) -> u32;
}

pub trait RangeOrdered {
    fn counter_start(self) -> u32;
    fn counter_end(self) -> u32;
}

pub trait OrderedValue<T>: Ordered
where
    T: Eq,
{
    fn value(self) -> T;
}

pub trait Scoped<T>
where
    T: Eq,
{
    fn contract_address(self) -> AztecAddress;
    fn inner(self) -> T;
}

pub trait Readable<T> {
    fn assert_match_read_request(self, read_request: T);
}

pub trait Inner<T> {
    fn inner(self) -> T;
}
