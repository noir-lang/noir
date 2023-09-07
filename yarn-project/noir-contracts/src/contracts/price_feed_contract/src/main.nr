mod asset;

contract PriceFeed {
    use dep::std::option::Option;
    use dep::aztec::{
        context::{PrivateContext, PublicContext},
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
        fn init(
            private_context: Option<&mut PrivateContext>,
            public_context: Option<&mut PublicContext>,
        ) -> pub Self {
            Storage {
                assets: Map::new(
                    private_context,
                    public_context,
                    1,
                    |private_context, public_context, slot| {
                        PublicState::new(
                            private_context,
                            public_context,
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
        let storage = Storage::init(Option::none(), Option::some(&mut context));
        let asset = storage.assets.at(asset_id);
        asset.write(Asset {price: price});

        1
    }

    #[aztec(public)]
    fn get_price(
        asset_id: Field,
    ) -> Asset {
        let storage = Storage::init(Option::none(), Option::some(&mut context));

        storage.assets.at(asset_id).read()
    }

    unconstrained fn fetch_price(
        assetId: Field,
    ) -> Asset {
        let storage = Storage::init(Option::none(), Option::none());
        storage.assets.at(assetId).read()
    }
}
