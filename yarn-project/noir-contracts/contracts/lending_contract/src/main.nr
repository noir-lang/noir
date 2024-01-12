mod asset;
mod interest_math;
mod helpers;
mod interfaces;

// Single asset CDP contract.
// Shoving re-entries up the ass.
// TODO's:
// - Use asset address instead of 0. We only use 0, as there is only one collateral asset :shrug:.
// - Update accumulator should be for specific asset, just abusing only 1 asset atm.
// - A way to repay all debt at once
// - Liquidations
contract Lending {
    use dep::aztec::protocol_types::{
        abis::function_selector::FunctionSelector,
        address::AztecAddress,
    };
    use dep::safe_math::SafeU120;
    use dep::std::option::Option;
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        state_vars::{
            map::Map,
            public_state::PublicState,
        },
        types::type_serialization::{
            field_serialization::{FieldSerializationMethods, FIELD_SERIALIZED_LEN},
            TypeSerializationInterface,
            address_serialization::{AddressSerializationMethods, AZTEC_ADDRESS_SERIALIZED_LEN},
        },
    };
    use crate::asset::{ASSET_SERIALIZED_LEN, Asset, AssetSerializationMethods};
    use crate::interest_math::compute_multiplier;
    use crate::helpers::{covered_by_collateral, DebtReturn, debt_updates, debt_value, compute_identifier};
    use crate::interfaces::{Token, Lending, PriceFeed};

    // Storage structure, containing all storage, and specifying what slots they use.
    struct Storage {
        collateral_asset: PublicState<AztecAddress, 1>,
        stable_coin: PublicState<AztecAddress, 1>,
        assets: Map<Field, PublicState<Asset, ASSET_SERIALIZED_LEN>>,
        collateral: Map<AztecAddress, PublicState<Field, FIELD_SERIALIZED_LEN>>,
        static_debt: Map<AztecAddress, PublicState<Field, FIELD_SERIALIZED_LEN>>, // abusing keys very heavily
    }

    impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                collateral_asset: PublicState::new(
                    context,
                    1,
                    AddressSerializationMethods,
                ),
                stable_coin: PublicState::new(
                    context,
                    2,
                    AddressSerializationMethods,
                ),
                assets: Map::new(
                    context,
                    3,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            AssetSerializationMethods,
                        )
                    },
                ),
                collateral: Map::new(
                    context,
                    4,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            FieldSerializationMethods,
                        )
                    },
                ),
                static_debt: Map::new(
                    context,
                    5,
                    |context, slot| {
                        PublicState::new(
                            context,
                            slot,
                            FieldSerializationMethods,
                        )
                    },
                ),
            }
        }
    }

    struct Position {
        collateral: Field,
        static_debt: Field,
        debt: Field,
    }

    // Constructs the contract.
    #[aztec(private)]
    fn constructor(
    ) {}

    #[aztec(public)]
    fn init(
        oracle: AztecAddress,
        loan_to_value: Field,
        collateral_asset: AztecAddress,
        stable_coin: AztecAddress
    ) {
        let asset_loc = storage.assets.at(0);
        let asset = asset_loc.read();

        assert(loan_to_value as u120 <= 10000);
        assert(asset.last_updated_ts == 0);
        assert(asset.interest_accumulator == 0);

        asset_loc.write(
            Asset {
            interest_accumulator: 1000000000,
            last_updated_ts: context.timestamp() as u120,
            loan_to_value: loan_to_value as u120,
            oracle
        }
        );

        storage.collateral_asset.write(collateral_asset);
        storage.stable_coin.write(stable_coin);
    }

    // Create a position.
    #[aztec(public)]
    fn update_accumulator() -> Asset {
        let asset_loc = storage.assets.at(0);
        let mut asset = asset_loc.read();

        let dt: SafeU120 = SafeU120 { value: context.timestamp() as u120 }.sub(SafeU120 { value: asset.last_updated_ts });

        // Only update if time has passed.
        if (!dt.is_zero()) {
            let precision: SafeU120 = SafeU120 { value: 1000000000 };
            let rate_per_second: u120 = 1268391679; // 4% yearly rate / (60 * 60 * 24 * 365)
            // if rate_per_second < sqrt(WAD) our approx is eq precision + rate * dt
            let multiplier = compute_multiplier(rate_per_second, dt);

            // accumulator *= multiplier, and multiplier >= 1
            asset.interest_accumulator = SafeU120{value: asset.interest_accumulator}.mul_div(multiplier, precision).value;
            asset.last_updated_ts = context.timestamp() as u120;

            asset_loc.write(asset);
        }

        asset
    }

    #[aztec(private)]
    fn deposit_private(
        from: AztecAddress,
        amount: Field,
        nonce: Field,
        secret: Field,
        on_behalf_of: Field,
        collateral_asset: AztecAddress
    ) {
        let on_behalf_of = compute_identifier(secret, on_behalf_of, context.msg_sender().to_field());
        let _res = Token::at(collateral_asset).unshield(&mut context, from, context.this_address(), amount, nonce);
        // _deposit(on_behalf_of, amount, collateral_asset)
        let selector = FunctionSelector::from_signature("_deposit((Field),Field,(Field))");
        context.call_public_function(
            context.this_address(),
            selector,
            [on_behalf_of, amount, collateral_asset.to_field()]
        );
    }

    #[aztec(public)]
    fn deposit_public(amount: Field, nonce: Field, on_behalf_of: Field, collateral_asset: AztecAddress) {
        Token::at(collateral_asset).transfer_public(
            context,
            context.msg_sender(),
            context.this_address(),
            amount,
            nonce
        );
        let selector = FunctionSelector::from_signature("_deposit((Field),Field,(Field))");
        context.call_public_function(
            context.this_address(),
            selector,
            [on_behalf_of, amount, collateral_asset.to_field()]
        );
    }

    #[aztec(public)]
    internal fn _deposit(owner: AztecAddress, amount: Field, collateral_asset: AztecAddress) {
        let _asset = Lending::at(context.this_address()).update_accumulator(context);

        let coll_asset = storage.collateral_asset.read();
        assert(coll_asset.eq(collateral_asset));

        let coll_loc = storage.collateral.at(owner);
        let collateral = coll_loc.read();
        coll_loc.write(collateral + amount);
    }

    #[aztec(private)]
    fn withdraw_private(secret: Field, to: AztecAddress, amount: Field) {
        let on_behalf_of = compute_identifier(secret, 0, context.msg_sender().to_field());
        let selector = FunctionSelector::from_signature("_withdraw((Field),(Field),Field)");
        context.call_public_function(
            context.this_address(),
            selector,
            [on_behalf_of, to.to_field(), amount]
        );
    }

    #[aztec(public)]
    fn withdraw_public(to: AztecAddress, amount: Field) {
        let selector = FunctionSelector::from_signature("_withdraw((Field),(Field),Field)");
        context.call_public_function(
            context.this_address(),
            selector,
            [context.msg_sender().to_field(), to.to_field(), amount]
        );
    }

    #[aztec(public)]
    internal fn _withdraw(owner: AztecAddress, recipient: AztecAddress, amount: Field) {
        let asset = Lending::at(context.this_address()).update_accumulator(context);
        let price = PriceFeed::at(asset.oracle).get_price(context);

        let coll_loc = storage.collateral.at(owner);
        let collateral: Field = coll_loc.read();

        let debt_loc = storage.static_debt.at(owner);
        let static_debt: Field = debt_loc.read();

        // debt_covered will revert if decrease would leave insufficient collateral to cover debt.
        // or trying to remove more collateral than available
        let debt_covered = covered_by_collateral(
            price,
            asset.loan_to_value,
            collateral as u120,
            0,
            amount as u120
        );
        let debt_returns = debt_updates(asset.interest_accumulator, static_debt as u120, 0, 0);

        assert(debt_returns.debt_value < debt_covered);

        coll_loc.write(collateral - amount);

        // @todo @LHerskind Support both shielding and transfers (for now just transfer)
        let collateral_asset = storage.collateral_asset.read();
        Token::at(collateral_asset).transfer_public(context, context.this_address(), recipient, amount, 0);
    }

    #[aztec(private)]
    fn borrow_private(secret: Field, to: AztecAddress, amount: Field) {
        let on_behalf_of = compute_identifier(secret, 0, context.msg_sender().to_field());
        let selector = FunctionSelector::from_signature("_borrow((Field),(Field),Field)");
        context.call_public_function(
            context.this_address(),
            selector,
            [on_behalf_of, to.to_field(), amount]
        );
    }

    #[aztec(public)]
    fn borrow_public(to: AztecAddress, amount: Field) {
        let selector = FunctionSelector::from_signature("_borrow((Field),(Field),Field)");
        context.call_public_function(
            context.this_address(),
            selector,
            [context.msg_sender().to_field(), to.to_field(), amount]
        );
    }

    #[aztec(public)]
    internal fn _borrow(owner: AztecAddress, to: AztecAddress, amount: Field) {
        let asset = Lending::at(context.this_address()).update_accumulator(context);
        let price = PriceFeed::at(asset.oracle).get_price(context);

        // Fetch collateral and static_debt, compute health of current position
        let collateral = storage.collateral.at(owner).read() as u120;
        let static_debt = storage.static_debt.at(owner).read() as u120;

        let debt_covered = covered_by_collateral(price, asset.loan_to_value, collateral, 0, 0);
        let debt_returns = debt_updates(asset.interest_accumulator, static_debt, amount as u120, 0);

        assert(debt_returns.debt_value < debt_covered);

        storage.static_debt.at(owner).write(debt_returns.static_debt as Field);

        // @todo @LHerskind Need to support both private and public minting.
        let stable_coin = storage.stable_coin.read();
        Token::at(stable_coin).mint_public(context, to, amount);
    }

    #[aztec(private)]
    fn repay_private(
        from: AztecAddress,
        amount: Field,
        nonce: Field,
        secret: Field,
        on_behalf_of: Field,
        stable_coin: AztecAddress
    ) {
        let on_behalf_of = compute_identifier(secret, on_behalf_of, context.msg_sender().to_field());
        let _res = Token::at(stable_coin).burn(&mut context, from, amount, nonce);
        let selector = FunctionSelector::from_signature("_repay((Field),Field,(Field))");
        context.call_public_function(
            context.this_address(),
            selector,
            [on_behalf_of, amount, stable_coin.to_field()]
        );
    }

    #[aztec(public)]
    fn repay_public(amount: Field, nonce: Field, owner: AztecAddress, stable_coin: AztecAddress) {
        Token::at(stable_coin).burn_public(context, context.msg_sender(), amount, nonce);
        let selector = FunctionSelector::from_signature("_repay((Field),Field,(Field))");
        context.call_public_function(
            context.this_address(),
            selector,
            [owner.to_field(), amount, stable_coin.to_field()]
        );
    }

    #[aztec(public)]
    internal fn _repay(owner: AztecAddress, amount: Field, stable_coin: AztecAddress) {
        let asset = Lending::at(context.this_address()).update_accumulator(context);

        // To ensure that private is using the correct token.
        assert(stable_coin.eq(storage.stable_coin.read()));

        let static_debt = storage.static_debt.at(owner).read() as u120;
        let debt_returns = debt_updates(asset.interest_accumulator, static_debt, 0, amount as u120);

        storage.static_debt.at(owner).write(debt_returns.static_debt as Field);
    }

    unconstrained fn get_asset(assetId: Field) -> pub Asset {
        storage.assets.at(assetId).read()
    }

    unconstrained fn get_position(owner: AztecAddress) -> pub Position {
        let collateral = storage.collateral.at(owner).read();
        let static_debt = storage.static_debt.at(owner).read();
        let asset = storage.assets.at(0).read();
        let debt = debt_value(static_debt as u120, asset.interest_accumulator as u120) as Field;
        Position { collateral, static_debt, debt }
    }

    unconstrained fn get_assets() -> pub [AztecAddress; 2] {
        [storage.collateral_asset.read(), storage.stable_coin.read()]
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
