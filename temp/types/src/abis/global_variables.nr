use crate::{
    abis::gas_fees::GasFees,
    address::{AztecAddress, EthAddress},
    constants::GLOBAL_VARIABLES_LENGTH,
    traits::{Deserialize, Empty, FromField, Serialize, ToField},
    utils::reader::Reader,
};

// docs:start:global-variables
pub struct GlobalVariables {
    pub chain_id: Field,
    pub version: Field,
    pub block_number: Field,
    pub slot_number: Field,
    pub timestamp: u64,
    pub coinbase: EthAddress,
    pub fee_recipient: AztecAddress,
    pub gas_fees: GasFees,
}
// docs:end:global-variables

impl GlobalVariables {
    pub fn is_empty(self) -> bool {
        (self.chain_id == 0)
            & (self.version == 0)
            & (self.block_number == 0)
            & (self.slot_number == 0)
            & (self.timestamp == 0)
            & (self.coinbase.is_zero())
            & (self.fee_recipient.is_zero())
            & (self.gas_fees.is_empty())
    }
}

impl Serialize<GLOBAL_VARIABLES_LENGTH> for GlobalVariables {
    fn serialize(self) -> [Field; GLOBAL_VARIABLES_LENGTH] {
        let mut serialized: BoundedVec<Field, GLOBAL_VARIABLES_LENGTH> = BoundedVec::new();

        serialized.push(self.chain_id);
        serialized.push(self.version);
        serialized.push(self.block_number);
        serialized.push(self.slot_number);
        serialized.push(self.timestamp as Field);
        serialized.push(self.coinbase.to_field());
        serialized.push(self.fee_recipient.to_field());
        serialized.extend_from_array(self.gas_fees.serialize());

        serialized.storage()
    }
}

impl Deserialize<GLOBAL_VARIABLES_LENGTH> for GlobalVariables {
    fn deserialize(serialized: [Field; GLOBAL_VARIABLES_LENGTH]) -> GlobalVariables {
        let mut reader = Reader::new(serialized);
        GlobalVariables {
            chain_id: reader.read(),
            version: reader.read(),
            block_number: reader.read(),
            slot_number: reader.read(),
            timestamp: reader.read() as u64,
            coinbase: EthAddress::from_field(reader.read()),
            fee_recipient: AztecAddress::from_field(reader.read()),
            gas_fees: reader.read_struct(GasFees::deserialize),
        }
    }
}

impl Eq for GlobalVariables {
    fn eq(self, other: GlobalVariables) -> bool {
        (self.chain_id == other.chain_id)
            & (self.version == other.version)
            & (self.block_number == other.block_number)
            & (self.slot_number == other.slot_number)
            & (self.timestamp == other.timestamp)
            & (self.coinbase == other.coinbase)
            & (self.fee_recipient == other.fee_recipient)
            & (self.gas_fees == other.gas_fees)
    }
}

impl Empty for GlobalVariables {
    fn empty() -> Self {
        Self {
            chain_id: 0,
            version: 0,
            block_number: 0,
            slot_number: 0,
            timestamp: 0,
            coinbase: EthAddress::empty(),
            fee_recipient: AztecAddress::empty(),
            gas_fees: GasFees::empty(),
        }
    }
}

#[test]
fn serialization_of_empty() {
    let vars = GlobalVariables::empty();
    let _serialized = vars.serialize();
    let _deserialized = GlobalVariables::deserialize(_serialized);
}
