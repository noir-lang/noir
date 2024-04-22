use dep::aztec::protocol_types::traits::{Serialize, Deserialize};

struct Asset {
    price: U128,
}

global ASSET_SERIALIZED_LEN: Field = 1;

impl Serialize<ASSET_SERIALIZED_LEN> for Asset {
    fn serialize(asset: Asset) -> [Field; ASSET_SERIALIZED_LEN] {
        [asset.price.to_field()]
    }
}

impl Deserialize<ASSET_SERIALIZED_LEN> for Asset {
    fn deserialize(fields: [Field; ASSET_SERIALIZED_LEN]) -> Asset {
        let price = U128::from_integer(fields[0]);
        Asset { price }
    }
}
