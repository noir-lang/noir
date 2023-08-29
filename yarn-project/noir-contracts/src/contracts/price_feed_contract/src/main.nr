mod storage;

contract PriceFeed {
    use dep::aztec::{
        abi,
        abi::{
            PrivateContextInputs,
            PublicContextInputs,
            Hasher
        }
    };
    use dep::aztec::context::{
        PrivateContext,
        PublicContext
    };
    use crate::storage::{Storage, Asset};

    #[aztec(private)]
    fn constructor(){}

    #[aztec(public)]
    open fn set_price(
        asset_id: Field,
        price: u120,
    ) {
        let storage = Storage::init();
        let asset = storage.assets.at(asset_id);
        asset.write(Asset {price: price});

        context.return_values.push(1);
    }

    #[aztec(public)]
    open fn get_price(
        asset_id: Field,
    ) {
        let storage = Storage::init();
        context.return_values.push_array(storage.assets.at(asset_id).read().serialise());
    }

    unconstrained fn fetch_price(
        assetId: Field,
    ) -> Asset {
        let storage = Storage::init();
        storage.assets.at(assetId).read()
    }
}
