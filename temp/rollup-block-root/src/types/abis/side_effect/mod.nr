pub(crate) mod counted;
pub(crate) mod scoped;

pub(crate) use counted::Counted;

use crate::types::address::AztecAddress;

pub(crate) trait Ordered {
    fn counter(self) -> u32;
}

pub(crate) trait RangeOrdered {
    fn counter_start(self) -> u32;
    fn counter_end(self) -> u32;
}

pub(crate) trait OrderedValue<T>: Ordered
where
    T: Eq,
{
    fn value(self) -> T;
}

pub(crate) trait Scoped<T>
where
    T: Eq,
{
    fn contract_address(self) -> AztecAddress;
    fn inner(self) -> T;
}

pub(crate) trait Readable<T> {
    fn assert_match_read_request(self, read_request: T);
}

pub(crate) trait Inner<T> {
    fn inner(self) -> T;
}
