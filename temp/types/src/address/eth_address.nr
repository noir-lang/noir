use crate::{
    constants::ETH_ADDRESS_LENGTH,
    traits::{Deserialize, Empty, Packable, Serialize, ToField},
};

pub struct EthAddress {
    inner: Field,
}

impl Eq for EthAddress {
    fn eq(self, other: Self) -> bool {
        self.to_field() == other.to_field()
    }
}

impl Empty for EthAddress {
    fn empty() -> Self {
        Self { inner: 0 }
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
        EthAddress::from_field(fields[0])
    }
}

impl Packable<ETH_ADDRESS_LENGTH> for EthAddress {
    fn pack(self) -> [Field; ETH_ADDRESS_LENGTH] {
        self.serialize()
    }

    fn unpack(fields: [Field; ETH_ADDRESS_LENGTH]) -> Self {
        Self::deserialize(fields)
    }
}

impl EthAddress {
    pub fn zero() -> Self {
        Self { inner: 0 }
    }

    pub fn from_field(field: Field) -> Self {
        field.assert_max_bit_size::<160>();
        Self { inner: field }
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }

    pub fn to_be_bytes(self) -> [u8; 20] {
        let field_bytes: [u8; 32] = self.inner.to_be_bytes();
        let mut bytes = [0; 20];
        for i in 0..20 {
            bytes[i] = field_bytes[i + 12];
        }
        bytes
    }
}
