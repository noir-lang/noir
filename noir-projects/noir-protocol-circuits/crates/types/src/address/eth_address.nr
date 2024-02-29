use crate::{
    constants::ETH_ADDRESS_LENGTH, hash::pedersen_hash,
    traits::{Empty, ToField, Serialize, Deserialize}, utils
};

struct EthAddress{
    inner : Field
}

impl Eq for EthAddress {
    fn eq(self, other : Self) -> bool {
        self.to_field() == other.to_field()
    }
}

impl Empty for EthAddress {
    fn empty() -> Self {
        Self {
            inner : 0
        }
    }
}

impl ToField for EthAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl Serialize<ETH_ADDRESS_LENGTH> for EthAddress {
    fn serialize(self: Self) -> [Field; ETH_ADDRESS_LENGTH] {
        [self.inner]
    }
}

impl Deserialize<ETH_ADDRESS_LENGTH> for EthAddress {
    fn deserialize(fields: [Field; ETH_ADDRESS_LENGTH]) -> Self {
        Self {
            inner: fields[0]
        }
    }
}

impl EthAddress {
    pub fn zero() -> Self {
        Self { inner: 0 }
    }

    pub fn from_field(field: Field) -> Self {
        Self { inner: field }
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }

    pub fn conditional_assign(predicate: bool, lhs: Self, rhs: Self) -> Self {
        let result = utils::conditional_assign(predicate, rhs.to_field(), lhs.to_field());
        Self { inner: result }
    }
}
