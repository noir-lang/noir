use dep::aztec::types::type_serialization::TypeSerializationInterface;

// Struct to be used to represent "totals". Generally, there should be one per asset.
// It stores the global values that are shared among all users, such as an accumulator
// and last time it was updated.
// In practice, it should also point to an oracle and have more fields related to
// loan to value ratios and other things, but we did not have enough reads/writes for this.
struct Asset {
    interest_accumulator: u120,
    last_updated_ts: u120,
    loan_to_value: u120,
    oracle_address: Field,
}

global ASSET_SERIALIZED_LEN: Field = 4;

// Right now we are wasting so many writes. If changing last_updated_ts
// we will end up rewriting all of them, wasting writes.
fn deserializeAsset(fields: [Field; ASSET_SERIALIZED_LEN]) -> Asset {
    Asset {
        interest_accumulator: fields[0] as u120,
        last_updated_ts: fields[1] as u120,
        loan_to_value: fields[2] as u120,
        oracle_address: fields[3],
    }
}

fn serializeAsset(asset: Asset) -> [Field; ASSET_SERIALIZED_LEN] {
    [
        asset.interest_accumulator as Field,
        asset.last_updated_ts as Field,
        asset.loan_to_value as Field,
        asset.oracle_address,
    ]
}

impl Asset {
    fn serialize(self: Self) -> [Field; ASSET_SERIALIZED_LEN] {
        serializeAsset(self)
    }
}

global AssetSerializationMethods = TypeSerializationInterface {
    deserialize: deserializeAsset,
    serialize: serializeAsset,
};
