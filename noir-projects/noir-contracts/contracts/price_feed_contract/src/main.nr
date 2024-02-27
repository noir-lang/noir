mod asset;

contract PriceFeed {
    use dep::std::option::Option;
    use dep::aztec::{context::{PrivateContext, PublicContext, Context}, state_vars::{Map, PublicMutable}};
    use dep::aztec::protocol_types::address::AztecAddress;
    use crate::asset::Asset;

    // Storage structure, containing all storage, and specifying what slots they use.
    struct Storage {
        assets: Map<Field, PublicMutable<Asset>>,
    }

    #[aztec(private)]
    fn constructor() {}

    #[aztec(public)]
    fn set_price(asset_id: Field, price: Field) {
        let asset = storage.assets.at(asset_id);
        asset.write(Asset { price: U128::from_integer(price) });
    }

    #[aztec(public)]
    fn get_price(asset_id: Field) -> Asset {
        storage.assets.at(asset_id).read()
    }

    unconstrained fn fetch_price(assetId: Field) -> pub Asset {
        storage.assets.at(assetId).read()
    }
}
