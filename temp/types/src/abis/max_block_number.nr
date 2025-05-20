use crate::{constants::MAX_BLOCK_NUMBER_LENGTH, traits::{Deserialize, Empty, Serialize}};

pub struct MaxBlockNumber {
    _opt: Option<u32>,
}

impl Empty for MaxBlockNumber {
    fn empty() -> Self {
        Self { _opt: Option::none() }
    }
}

impl Eq for MaxBlockNumber {
    fn eq(self, other: Self) -> bool {
        self._opt == other._opt
    }
}

impl Serialize<MAX_BLOCK_NUMBER_LENGTH> for MaxBlockNumber {
    fn serialize(self) -> [Field; MAX_BLOCK_NUMBER_LENGTH] {
        [self._opt.is_some() as Field, self._opt.unwrap_unchecked() as Field]
    }
}

impl Deserialize<MAX_BLOCK_NUMBER_LENGTH> for MaxBlockNumber {
    fn deserialize(serialized: [Field; MAX_BLOCK_NUMBER_LENGTH]) -> MaxBlockNumber {
        MaxBlockNumber {
            _opt: if serialized[0] as bool {
                Option::some(serialized[1] as u32)
            } else {
                Option::none()
            },
        }
    }
}

impl MaxBlockNumber {
    pub fn new(max_block_number: u32) -> Self {
        Self { _opt: Option::some(max_block_number) }
    }

    pub fn is_none(self) -> bool {
        self._opt.is_none()
    }

    pub fn is_some(self) -> bool {
        self._opt.is_some()
    }

    pub fn unwrap(self) -> u32 {
        self._opt.unwrap()
    }

    pub fn unwrap_unchecked(self) -> u32 {
        self._opt.unwrap_unchecked()
    }

    pub fn min(lhs: MaxBlockNumber, rhs: MaxBlockNumber) -> MaxBlockNumber {
        if rhs.is_none() {
            lhs // lhs might also be none, but in that case both would be
        } else {
            MaxBlockNumber::min_with_u32(lhs, rhs.unwrap_unchecked())
        }
    }

    pub fn min_with_u32(lhs: MaxBlockNumber, rhs: u32) -> MaxBlockNumber {
        if lhs._opt.is_none() {
            MaxBlockNumber::new(rhs)
        } else {
            let lhs_value = lhs._opt.unwrap_unchecked();

            MaxBlockNumber::new(if lhs_value < rhs { lhs_value } else { rhs })
        }
    }
}

#[test]
fn serialization_of_empty() {
    let item = MaxBlockNumber::empty();
    let serialized = item.serialize();
    let deserialized = MaxBlockNumber::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn zeroed_is_none() {
    // Large parts of the kernel rely on zeroed to initialize structs. This conveniently matches what `default` does,
    // and though we should eventually move everything to use `default`, it's good to check for now that both are
    // equivalent.
    let a = MaxBlockNumber::empty();
    assert(a.is_none());
}

#[test]
fn serde_default() {
    let a = MaxBlockNumber::empty();
    let b = MaxBlockNumber::deserialize(a.serialize());
    assert(b.is_none());
}

#[test]
fn serde_some() {
    let a = MaxBlockNumber::new(13);
    let b = MaxBlockNumber::deserialize(a.serialize());
    assert_eq(b.unwrap(), 13);
}

#[test(should_fail)]
fn default_unwrap_panics() {
    let a = MaxBlockNumber::empty();
    let _ = a.unwrap();
}

#[test]
fn min_default_default() {
    let a = MaxBlockNumber::empty();
    let b = MaxBlockNumber::empty();

    assert(MaxBlockNumber::min(a, b).is_none());
}

#[test]
fn min_default_some() {
    let a = MaxBlockNumber::empty();
    let b = MaxBlockNumber::new(13);

    assert_eq(MaxBlockNumber::min(a, b).unwrap(), 13);
    assert_eq(MaxBlockNumber::min(b, a).unwrap(), 13);
}

#[test]
fn min_some_some() {
    let a = MaxBlockNumber::new(13);
    let b = MaxBlockNumber::new(42);

    assert_eq(MaxBlockNumber::min(a, b).unwrap(), 13);
    assert_eq(MaxBlockNumber::min(b, a).unwrap(), 13);
}

#[test]
fn min_with_u32_default() {
    let a = MaxBlockNumber::empty();
    let b = 42;

    assert_eq(MaxBlockNumber::min_with_u32(a, b).unwrap(), 42);
}

#[test]
fn min_with_u32_some() {
    let a = MaxBlockNumber::new(13);
    let b = 42;
    let c = 8;

    assert_eq(MaxBlockNumber::min_with_u32(a, b).unwrap(), 13);
    assert_eq(MaxBlockNumber::min_with_u32(a, c).unwrap(), 8);
}
