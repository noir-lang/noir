use dep::aztec::protocol_types::traits::{Serialize, Deserialize};

struct Asset {
    price: u120,
}

global ASSET_SERIALIZED_LEN: Field = 1;

impl Serialize<ASSET_SERIALIZED_LEN> for Asset {
    fn serialize(asset: Asset) -> [Field; ASSET_SERIALIZED_LEN] {
        [asset.price as Field]
    }
}

impl Deserialize<ASSET_SERIALIZED_LEN> for Asset {
    fn deserialize(fields: [Field; ASSET_SERIALIZED_LEN]) -> Asset {
        Asset { price: fields[0] as u120 }
    }
}
