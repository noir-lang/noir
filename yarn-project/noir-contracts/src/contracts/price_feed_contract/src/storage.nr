use dep::aztec::state_vars::map::Map;
use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::types::type_serialisation::TypeSerialisationInterface;
use dep::aztec::types::type_serialisation::field_serialisation::FieldSerialisationMethods;
use dep::aztec::types::type_serialisation::field_serialisation::FIELD_SERIALISED_LEN;
use dep::std::hash::pedersen;

struct Asset {
    price: u120,
}

global ASSET_SERIALISED_LEN: Field = 1;

fn deserialiseAsset(fields: [Field; ASSET_SERIALISED_LEN]) -> Asset {
    Asset {
        price: fields[0] as u120,
    }
}

fn serialiseAsset(asset: Asset) -> [Field; ASSET_SERIALISED_LEN] {
    [asset.price as Field]
}

impl Asset {
    fn serialize(self: Self) -> [Field; ASSET_SERIALISED_LEN] {
        serialiseAsset(self)
    }
}

global AssetSerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialiseAsset,
    serialise: serialiseAsset,
};

// Storage structure, containing all storage, and specifying what slots they use.
struct Storage {
    assets: Map<PublicState<Asset, ASSET_SERIALISED_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            assets: Map::new(1, |slot| PublicState::new(slot, AssetSerialisationMethods)),
        }
    }
}