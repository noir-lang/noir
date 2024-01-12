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
    use dep::aztec::protocol_types::address::AztecAddress;
    use crate::asset::{ASSET_SERIALIZED_LEN, Asset, AssetSerializationMethods};

    // Storage structure, containing all storage, and specifying what slots they use.
    struct Storage {
        assets: Map<Field, PublicState<Asset, ASSET_SERIALIZED_LEN>>,
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
                            AssetSerializationMethods,
                        )
                    },
                ),
            }
        }
    }

    #[aztec(private)]
    fn constructor() {}

    #[aztec(public)]
    fn set_price(asset_id: Field, price: u120) {
        let asset = storage.assets.at(asset_id);
        asset.write(Asset { price });
    }

    #[aztec(public)]
    fn get_price(asset_id: Field) -> Asset {
        storage.assets.at(asset_id).read()
    }

    unconstrained fn fetch_price(assetId: Field) -> pub Asset {
        storage.assets.at(assetId).read()
    }

    // TODO: remove this placeholder once https://github.com/AztecProtocol/aztec-packages/issues/2918 is implemented
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; 0]
    ) -> pub [Field; 4] {
        [0, 0, 0, 0]
    }
}
