use dep::aztec::types::type_serialisation::TypeSerialisationInterface;

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
