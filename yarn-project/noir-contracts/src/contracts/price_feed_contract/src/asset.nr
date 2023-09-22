use dep::aztec::types::type_serialization::TypeSerializationInterface;

struct Asset {
    price: u120,
}

global ASSET_SERIALIZED_LEN: Field = 1;

fn deserializeAsset(fields: [Field; ASSET_SERIALIZED_LEN]) -> Asset {
    Asset {
        price: fields[0] as u120,
    }
}

fn serializeAsset(asset: Asset) -> [Field; ASSET_SERIALIZED_LEN] {
    [asset.price as Field]
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
