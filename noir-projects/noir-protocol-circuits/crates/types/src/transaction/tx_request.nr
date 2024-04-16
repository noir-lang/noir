use crate::{
    address::AztecAddress, abis::function_data::FunctionData, abis::gas_settings::GasSettings,
    constants::{GENERATOR_INDEX__TX_REQUEST, TX_REQUEST_LENGTH}, hash::pedersen_hash,
    traits::{Hash, Serialize, Deserialize, Empty}, transaction::tx_context::TxContext,
    utils::reader::Reader
};

struct TxRequest {
    origin: AztecAddress,
    args_hash: Field,
    tx_context: TxContext,
    function_data: FunctionData,
    gas_settings: GasSettings,
}

impl Empty for TxRequest {
    fn empty() -> Self {
        TxRequest {
            origin: AztecAddress::empty(),
            args_hash: 0,
            tx_context: TxContext::empty(),
            function_data: FunctionData::empty(),
            gas_settings: GasSettings::empty(),
        }
    }
}

impl Eq for TxRequest {
    fn eq(self, other: Self) -> bool {
        (self.origin == other.origin) &
        (self.args_hash == other.args_hash) &
        (self.tx_context == other.tx_context) &
        (self.function_data == other.function_data) &
        (self.gas_settings == other.gas_settings)
    }
}

impl Hash for TxRequest {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__TX_REQUEST)
    }
}

impl Serialize<TX_REQUEST_LENGTH> for TxRequest {
    fn serialize(self) -> [Field; TX_REQUEST_LENGTH] {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut fields: BoundedVec<Field, TX_REQUEST_LENGTH> = BoundedVec::new();

        fields.push(self.origin.to_field());
        fields.extend_from_array(self.function_data.serialize());
        fields.push(self.args_hash);
        fields.extend_from_array(self.tx_context.serialize());
        fields.extend_from_array(self.gas_settings.serialize());

        assert_eq(fields.len(), TX_REQUEST_LENGTH);

        fields.storage
    }
}

impl Deserialize<TX_REQUEST_LENGTH> for TxRequest {
    fn deserialize(serialized: [Field; TX_REQUEST_LENGTH]) -> Self {
        let mut reader = Reader::new(serialized);

        let request = Self {
            origin: reader.read_struct(AztecAddress::deserialize),
            args_hash: reader.read(),
            tx_context: reader.read_struct(TxContext::deserialize),
            function_data: reader.read_struct(FunctionData::deserialize),
            gas_settings: reader.read_struct(GasSettings::deserialize),
        };

        reader.finish();
        request
    }
}

mod tests {
    use crate::{
        abis::{
        function_selector::FunctionSelector, function_data::FunctionData,
        gas_settings::{GasSettings, DimensionGasSettings}
    },
        address::{AztecAddress, EthAddress}, contract_class_id::ContractClassId,
        grumpkin_point::GrumpkinPoint, transaction::{tx_request::TxRequest, tx_context::TxContext}
    };

    #[test]
    fn serialization_of_empty() {
        let request = TxRequest::empty();
        let serialized = request.serialize();
        let deserialized = TxRequest::deserialize(serialized);
        assert(request.eq(deserialized));
    }

    #[test]
    fn compute_hash() {
        let tx_request = TxRequest {
            origin: AztecAddress::from_field(1),
            args_hash: 3,
            tx_context: TxContext { is_fee_payment_tx: false, is_rebate_payment_tx: false, chain_id: 0, version: 0 },
            function_data: FunctionData { selector: FunctionSelector::from_u32(2), is_private: true },
            gas_settings: GasSettings {
                da: DimensionGasSettings { gas_limit: 2, teardown_gas_limit: 1, max_fee_per_gas: 3 },
                l1: DimensionGasSettings { gas_limit: 2, teardown_gas_limit: 1, max_fee_per_gas: 3 },
                l2: DimensionGasSettings { gas_limit: 2, teardown_gas_limit: 1, max_fee_per_gas: 3 },
                inclusion_fee: 10
            }
        };
        // Value from tx_request.test.ts "compute hash" test
        let test_data_tx_request_hash = 0x03b678e327818eb368f9eac21839ee67a98968318f0dcd76c89d3fcf66af7257;
        assert(tx_request.hash() == test_data_tx_request_hash);
    }
}
