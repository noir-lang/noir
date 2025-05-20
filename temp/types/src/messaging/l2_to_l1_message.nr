use crate::{
    abis::side_effect::{Ordered, Scoped},
    address::{AztecAddress, EthAddress},
    constants::{L2_TO_L1_MESSAGE_LENGTH, SCOPED_L2_TO_L1_MESSAGE_LENGTH},
    traits::{Deserialize, Empty, Serialize, ToField},
    utils::{arrays::array_concat, reader::Reader},
};

// Note: Not to be confused with L2ToL1Msg in Solidity
pub struct L2ToL1Message {
    pub recipient: EthAddress,
    pub content: Field,
    pub counter: u32,
}

impl Ordered for L2ToL1Message {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Empty for L2ToL1Message {
    fn empty() -> Self {
        Self { recipient: EthAddress::empty(), content: 0, counter: 0 }
    }
}

impl Eq for L2ToL1Message {
    fn eq(self, other: Self) -> bool {
        (self.recipient == other.recipient)
            & (self.content == other.content)
            & (self.counter == other.counter)
    }
}

impl Serialize<L2_TO_L1_MESSAGE_LENGTH> for L2ToL1Message {
    fn serialize(self) -> [Field; L2_TO_L1_MESSAGE_LENGTH] {
        [self.recipient.to_field(), self.content, self.counter as Field]
    }
}

impl Deserialize<L2_TO_L1_MESSAGE_LENGTH> for L2ToL1Message {
    fn deserialize(values: [Field; L2_TO_L1_MESSAGE_LENGTH]) -> Self {
        Self {
            recipient: EthAddress::from_field(values[0]),
            content: values[1],
            counter: values[2] as u32,
        }
    }
}

impl L2ToL1Message {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedL2ToL1Message {
        ScopedL2ToL1Message { message: self, contract_address }
    }
}

pub struct ScopedL2ToL1Message {
    pub message: L2ToL1Message,
    pub contract_address: AztecAddress,
}

impl ScopedL2ToL1Message {
    pub fn expose_to_public(self) -> Self {
        // Hide the counter when exposing to public.
        Self {
            message: L2ToL1Message {
                recipient: self.message.recipient,
                content: self.message.content,
                counter: 0,
            },
            contract_address: self.contract_address,
        }
    }
}

impl Scoped<L2ToL1Message> for ScopedL2ToL1Message {
    fn inner(self) -> L2ToL1Message {
        self.message
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Ordered for ScopedL2ToL1Message {
    fn counter(self) -> u32 {
        self.message.counter
    }
}

impl Eq for ScopedL2ToL1Message {
    fn eq(self, other: ScopedL2ToL1Message) -> bool {
        (self.message == other.message) & (self.contract_address == other.contract_address)
    }
}

impl Empty for ScopedL2ToL1Message {
    fn empty() -> Self {
        ScopedL2ToL1Message {
            message: L2ToL1Message::empty(),
            contract_address: AztecAddress::empty(),
        }
    }
}

impl Serialize<SCOPED_L2_TO_L1_MESSAGE_LENGTH> for ScopedL2ToL1Message {
    fn serialize(self) -> [Field; SCOPED_L2_TO_L1_MESSAGE_LENGTH] {
        array_concat(self.message.serialize(), [self.contract_address.to_field()])
    }
}

impl Deserialize<SCOPED_L2_TO_L1_MESSAGE_LENGTH> for ScopedL2ToL1Message {
    fn deserialize(values: [Field; SCOPED_L2_TO_L1_MESSAGE_LENGTH]) -> Self {
        let mut reader = Reader::new(values);
        let res = Self {
            message: reader.read_struct(L2ToL1Message::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        res
    }
}

#[test]
fn serialization_of_empty_l2() {
    let item = L2ToL1Message::empty();
    let serialized = item.serialize();
    let deserialized = L2ToL1Message::deserialize(serialized);
    assert(item.eq(deserialized));
}

#[test]
fn serialization_of_empty_scoped_l2() {
    let item = ScopedL2ToL1Message::empty();
    let serialized = item.serialize();
    let deserialized = ScopedL2ToL1Message::deserialize(serialized);
    assert(item.eq(deserialized));
}
