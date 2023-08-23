use dep::aztec::state_vars::map::Map;
use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::types::type_serialisation::TypeSerialisationInterface;
use dep::aztec::types::type_serialisation::field_serialisation::FieldSerialisationMethods;
use dep::aztec::types::type_serialisation::field_serialisation::FIELD_SERIALISED_LEN;

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

global ASSET_SERIALISED_LEN: Field = 4;

// Right now we are wasting so many writes. If changing last_updated_ts
// we will end up rewriting all of them, wasting writes.
fn deserialiseAsset(fields: [Field; ASSET_SERIALISED_LEN]) -> Asset {
    Asset {
        interest_accumulator: fields[0] as u120,
        last_updated_ts: fields[1] as u120,
        loan_to_value: fields[2] as u120,
        oracle_address: fields[3],
    }
}

fn serialiseAsset(asset: Asset) -> [Field; ASSET_SERIALISED_LEN] {
    [asset.interest_accumulator as Field, asset.last_updated_ts as Field, asset.loan_to_value as Field, asset.oracle_address]
}

impl Asset {
    fn serialise (self: Self) -> [Field; ASSET_SERIALISED_LEN] {
        serialiseAsset(self)
    }
}

global AssetSerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialiseAsset,
    serialise: serialiseAsset,
};

// Storage structure, containing all storage, and specifying what slots they use.
struct Storage {
    collateral_asset: PublicState<Field, 1>,
    stable_coin: PublicState<Field, 1>,
    assets: Map<PublicState<Asset, ASSET_SERIALISED_LEN>>,
    collateral: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
    static_debt: Map<PublicState<Field, FIELD_SERIALISED_LEN>>, // abusing keys very heavily
}

impl Storage {
    fn init() -> Self {
        Storage {
            collateral_asset: PublicState::new(1, FieldSerialisationMethods),
            stable_coin: PublicState::new(2, FieldSerialisationMethods),
            assets: Map::new(3, |slot| PublicState::new(slot, AssetSerialisationMethods)),
            collateral: Map::new(4, |slot| PublicState::new(slot, FieldSerialisationMethods)),
            static_debt: Map::new(5, |slot| PublicState::new(slot, FieldSerialisationMethods)),
        }
    }
}