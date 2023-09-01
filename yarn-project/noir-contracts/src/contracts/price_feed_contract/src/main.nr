mod storage;

contract PriceFeed {
    use crate::storage::{Storage, Asset};

    #[aztec(private)]
    fn constructor(){}

    #[aztec(public)]
    fn set_price(
        asset_id: Field,
        price: u120,
    ) -> Field {
        let storage = Storage::init();
        let asset = storage.assets.at(asset_id);
        asset.write(Asset {price: price});

        1
    }

    #[aztec(public)]
    fn get_price(
        asset_id: Field,
    ) -> Asset {
        let storage = Storage::init();

        storage.assets.at(asset_id).read()
    }

    unconstrained fn fetch_price(
        assetId: Field,
    ) -> Asset {
        let storage = Storage::init();
        storage.assets.at(assetId).read()
    }
}
