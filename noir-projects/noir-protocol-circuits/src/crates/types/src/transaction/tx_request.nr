use crate::{
    address::AztecAddress, abis::function_data::FunctionData,
    constants::{GENERATOR_INDEX__TX_REQUEST, TX_REQUEST_LENGTH}, hash::pedersen_hash,
    traits::{Hash, Serialize, Deserialize}, transaction::tx_context::TxContext, utils::reader::Reader
};

struct TxRequest {
    origin: AztecAddress,
    args_hash: Field,
    tx_context: TxContext,
    function_data: FunctionData,
}

impl Eq for TxRequest {
    fn eq(self, other: Self) -> bool {
        (self.origin == other.origin) &
        (self.args_hash == other.args_hash) &
        (self.tx_context == other.tx_context) &
        (self.function_data == other.function_data)
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
        };

        reader.finish();
        request
    }
}

mod tests {
    use crate::{
        abis::{function_selector::FunctionSelector, function_data::FunctionData},
        address::{AztecAddress, EthAddress}, contract_class_id::ContractClassId,
        contrakt::contract_deployment_data::ContractDeploymentData, grumpkin_point::GrumpkinPoint,
        transaction::{tx_request::TxRequest, tx_context::TxContext}
    };

    #[test]
    fn serialization_of_empty() {
        // Assuming a zeroed initialization for simplicity
        let request: TxRequest = dep::std::unsafe::zeroed();
        let serialized = request.serialize();
        let deserialized = TxRequest::deserialize(serialized);
        assert(request.eq(deserialized));
    }

    #[test]
    fn compute_hash() {
        let tx_request = TxRequest {
            origin: AztecAddress::from_field(1),
            args_hash: 3,
            tx_context: TxContext {
                is_fee_payment_tx: false,
                is_rebate_payment_tx: false,
                is_contract_deployment_tx: true,
                contract_deployment_data: ContractDeploymentData {
                    public_key: GrumpkinPoint { x: 1, y: 2 },
                    initialization_hash: 1,
                    contract_class_id: ContractClassId::from_field(2),
                    contract_address_salt: 3,
                    portal_contract_address: EthAddress::from_field(1)
                },
                chain_id: 0,
                version: 0
            },
            function_data: FunctionData { selector: FunctionSelector::from_u32(2), is_internal: false, is_private: true, is_constructor: true }
        };
        // Value from tx_request.test.ts "compute hash" test
        assert(tx_request.hash() == 0x05e3f67a5e787faf63c85c28169a9d616b594c9799d9785b4f167d9cd7a71e9e);
    }
}
