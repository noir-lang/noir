mod asset;

contract PriceFeed {
    use dep::std::option::Option;
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        state_vars::{
            map::Map,
            public_state::PublicState,
        },
    };
    use crate::asset::{ASSET_SERIALISED_LEN, Asset, AssetSerialisationMethods};

    // Storage structure, containing all storage, and specifying what slots they use.
    struct Storage {
        assets: Map<PublicState<Asset, ASSET_SERIALISED_LEN>>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                assets: Map::new(
                    context,
                    1,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            AssetSerialisationMethods,
                        )
                    },
                ),
            }
        }
    }

    #[aztec(private)]
    fn constructor(){}

    #[aztec(public)]
    fn set_price(
        asset_id: Field,
        price: u120,
    ) -> Field {
        let storage = Storage::init(Context::public(&mut context));
        let asset = storage.assets.at(asset_id);
        asset.write(Asset {price: price});

        1
    }

    #[aztec(public)]
    fn get_price(
        asset_id: Field,
    ) -> Asset {
        let storage = Storage::init(Context::public(&mut context));

        storage.assets.at(asset_id).read()
    }

    unconstrained fn fetch_price(
        assetId: Field,
    ) -> Asset {
        let storage = Storage::init(Context::none());
        storage.assets.at(assetId).read()
    }
}
