use crate::{abis::read_request::ScopedReadRequest, address::AztecAddress};
use dep::std::cmp::Eq;

trait Ordered {
    fn counter(self) -> u32;
}

trait RangeOrdered {
    fn counter_start(self) -> u32;
    fn counter_end(self) -> u32;
}

trait OrderedValue<T> where T: Eq {
    fn value(self) -> T;
    fn counter(self) -> u32;
}

trait Scoped<T> where T: Eq {
    fn contract_address(self) -> AztecAddress;
    fn inner(self) -> T;
}

trait Positioned {
    // Like a storage slot
    fn position(self) -> Field;
}

trait Readable {
    fn assert_match_read_request(self, read_request: ScopedReadRequest);
}

