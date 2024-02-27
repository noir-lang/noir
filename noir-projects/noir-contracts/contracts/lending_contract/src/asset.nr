use dep::aztec::protocol_types::{address::AztecAddress, traits::{Deserialize, Serialize}};

// Struct to be used to represent "totals". Generally, there should be one per asset.
// It stores the global values that are shared among all users, such as an accumulator
// and last time it was updated.
// In practice, it should also point to an oracle and have more fields related to
// loan to value ratios and other things, but we did not have enough reads/writes for this.
struct Asset {
    interest_accumulator: U128,
    last_updated_ts: u64,
    loan_to_value: U128,
    oracle: AztecAddress,
}

global ASSET_SERIALIZED_LEN: Field = 4;

impl Serialize<ASSET_SERIALIZED_LEN> for Asset {
    fn serialize(asset: Asset) -> [Field; ASSET_SERIALIZED_LEN] {
        [
            asset.interest_accumulator.to_integer(),
            asset.last_updated_ts as Field,
            asset.loan_to_value.to_integer(),
            asset.oracle.to_field()
        ]
    }
}

impl Deserialize<ASSET_SERIALIZED_LEN> for Asset {
    // Right now we are wasting so many writes. If changing last_updated_ts
    // we will end up rewriting all of them, wasting writes.
    fn deserialize(fields: [Field; ASSET_SERIALIZED_LEN]) -> Asset {
        let interest_accumulator = U128::from_integer(fields[0]);
        let last_updated_ts = fields[1] as u64;
        let loan_to_value = U128::from_integer(fields[2]);
        let oracle = AztecAddress::from_field(fields[3]);

        Asset {
            interest_accumulator,
            last_updated_ts,
            loan_to_value,
            oracle,
        }
    }
}
