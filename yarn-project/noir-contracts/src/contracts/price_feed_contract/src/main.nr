mod storage;

contract PriceFeed {
    use dep::aztec::{
        abi,
        abi::PrivateContextInputs,
        abi::PublicContextInputs
    };
    use dep::aztec::context::{
        PrivateContext,
        PublicContext
    };
    use crate::storage::{Storage, Asset};

    fn constructor(
        inputs: PrivateContextInputs
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        PrivateContext::new(inputs, 0).finish()
    }

    open fn set_price(
        inputs: PublicContextInputs,
        asset_id: Field,
        price: u120,
    ) -> pub abi::PublicCircuitPublicInputs  {
        let mut context = PublicContext::new(inputs, abi::hash_args([asset_id, price as Field]));

        let storage = Storage::init();
        let asset = storage.assets.at(asset_id);
        asset.write(Asset {price: price});

        context.return_values.push(1);
        context.finish()
    }

    open fn get_price(
        inputs: PublicContextInputs,
        asset_id: Field,
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([asset_id]));
        let storage = Storage::init();
        context.return_values.push_array(storage.assets.at(asset_id).read().serialise());
        context.finish()
    }

    unconstrained fn fetch_price(
        assetId: Field,
    ) -> Asset {
        let storage = Storage::init();
        storage.assets.at(assetId).read()
    }
}
