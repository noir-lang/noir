mod asset;
mod interest_math;
mod helpers;

// Single asset CDP contract.
// Shoving re-entries up the ass.
// TODO's:
// - Use asset address instead of 0. We only use 0, as there is only one collateral asset :shrug:.
// - Update accumulator should be for specific asset, just abusing only 1 asset atm.
// - A way to repay all debt at once
// - Liquidations
contract Lending {
    use dep::aztec::prelude::{FunctionSelector, AztecAddress, PrivateContext, Map, PublicMutable};
    use dep::aztec::context::{PublicContext, Context, gas::GasOpts};

    use crate::asset::Asset;
    use crate::interest_math::compute_multiplier;
    use crate::helpers::{covered_by_collateral, DebtReturn, debt_updates, debt_value, compute_identifier};
    use dep::token::Token;
    use dep::price_feed::PriceFeed;

    // Storage structure, containing all storage, and specifying what slots they use.
    #[aztec(storage)]
    struct Storage {
        collateral_asset: PublicMutable<AztecAddress>,
        stable_coin: PublicMutable<AztecAddress>,
        assets: Map<Field, PublicMutable<Asset>>,
        collateral: Map<AztecAddress, PublicMutable<Field>>,
        static_debt: Map<AztecAddress, PublicMutable<Field>>, // abusing keys very heavily
    }

    struct Position {
        collateral: Field,
        static_debt: Field,
        debt: Field,
    }

    // Constructs the contract.
    #[aztec(private)]
    #[aztec(initializer)]
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
        let asset: Asset = asset_loc.read();

        let loan_to_value = U128::from_integer(loan_to_value);

        assert(loan_to_value <= U128::from_integer(10000));
        assert(asset.last_updated_ts == 0);
        assert(asset.interest_accumulator == U128::from_integer(0));

        let last_updated_ts = context.timestamp();

        asset_loc.write(
            Asset { interest_accumulator: U128::from_integer(1000000000), last_updated_ts, loan_to_value, oracle }
        );

        storage.collateral_asset.write(collateral_asset);
        storage.stable_coin.write(stable_coin);
    }

    // Create a position.
    #[aztec(public)]
    fn update_accumulator() -> Asset {
        let asset_loc = storage.assets.at(0);
        let mut asset: Asset = asset_loc.read();

        let timestamp = context.timestamp();
        let dt = timestamp - asset.last_updated_ts;

        // Only update if time has passed.
        if !(dt == 0) {
            let precision = U128::from_integer(1000000000);
            let rate_per_second = U128::from_integer(1268391679); // 4% yearly rate / (60 * 60 * 24 * 365)
            // if rate_per_second < sqrt(WAD) our approx is eq precision + rate * dt
            let multiplier = compute_multiplier(rate_per_second, dt);

            // accumulator *= multiplier, and multiplier >= 1
            asset.interest_accumulator = (asset.interest_accumulator * multiplier) / precision;
            asset.last_updated_ts = context.timestamp();

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
        let _res = Token::at(collateral_asset).unshield(from, context.this_address(), amount, nonce).call(&mut context);
        Lending::at(context.this_address())._deposit(
            AztecAddress::from_field(on_behalf_of),
            amount,
            collateral_asset
        ).enqueue(&mut context);
    }

    #[aztec(public)]
    fn deposit_public(amount: Field, nonce: Field, on_behalf_of: Field, collateral_asset: AztecAddress) {
        let _ = Token::at(collateral_asset).transfer_public(context.msg_sender(), context.this_address(), amount, nonce).call(&mut context);
        let _ = Lending::at(context.this_address())._deposit(
            AztecAddress::from_field(on_behalf_of),
            amount,
            collateral_asset
        ).call(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _deposit(owner: AztecAddress, amount: Field, collateral_asset: AztecAddress) {
        let _asset = Lending::at(context.this_address()).update_accumulator().call(&mut context);

        let coll_asset = storage.collateral_asset.read();
        assert(coll_asset.eq(collateral_asset));

        let coll_loc = storage.collateral.at(owner);
        let collateral = coll_loc.read();
        coll_loc.write(collateral + amount);
    }

    #[aztec(private)]
    fn withdraw_private(secret: Field, to: AztecAddress, amount: Field) {
        let on_behalf_of = compute_identifier(secret, 0, context.msg_sender().to_field());
        Lending::at(context.this_address())._withdraw(AztecAddress::from_field(on_behalf_of), to, amount).enqueue(&mut context);
    }

    #[aztec(public)]
    fn withdraw_public(to: AztecAddress, amount: Field) {
        let _ = Lending::at(context.this_address())._withdraw(context.msg_sender(), to, amount).call(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _withdraw(owner: AztecAddress, recipient: AztecAddress, amount: Field) {
        let asset = Lending::at(context.this_address()).update_accumulator().call(&mut context);
        let price = PriceFeed::at(asset.oracle).get_price(0).call(&mut context).price;

        let coll_loc = storage.collateral.at(owner);
        let collateral: Field = coll_loc.read();

        let debt_loc = storage.static_debt.at(owner);
        let static_debt: Field = debt_loc.read();

        // debt_covered will revert if decrease would leave insufficient collateral to cover debt.
        // or trying to remove more collateral than available
        let debt_covered = covered_by_collateral(
            price,
            asset.loan_to_value,
            U128::from_integer(collateral),
            U128::from_integer(0),
            U128::from_integer(amount)
        );
        let debt_returns = debt_updates(
            asset.interest_accumulator,
            U128::from_integer(static_debt),
            U128::from_integer(0),
            U128::from_integer(0)
        );

        assert(debt_returns.debt_value < debt_covered);

        coll_loc.write(collateral - amount);

        // @todo @LHerskind Support both shielding and transfers (for now just transfer)
        let collateral_asset = storage.collateral_asset.read();
        let _ = Token::at(collateral_asset).transfer_public(context.this_address(), recipient, amount, 0).call(&mut context);
    }

    #[aztec(private)]
    fn borrow_private(secret: Field, to: AztecAddress, amount: Field) {
        let on_behalf_of = compute_identifier(secret, 0, context.msg_sender().to_field());
        let _ = Lending::at(context.this_address())._borrow(AztecAddress::from_field(on_behalf_of), to, amount).enqueue(&mut context);
    }

    #[aztec(public)]
    fn borrow_public(to: AztecAddress, amount: Field) {
        let _ = Lending::at(context.this_address())._borrow(context.msg_sender(), to, amount).call(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _borrow(owner: AztecAddress, to: AztecAddress, amount: Field) {
        let asset = Lending::at(context.this_address()).update_accumulator().call(&mut context);
        let price = PriceFeed::at(asset.oracle).get_price(0).call(&mut context).price;

        // Fetch collateral and static_debt, compute health of current position
        let collateral = U128::from_integer(storage.collateral.at(owner).read());
        let static_debt = U128::from_integer(storage.static_debt.at(owner).read());

        let debt_covered = covered_by_collateral(
            price,
            asset.loan_to_value,
            collateral,
            U128::from_integer(0),
            U128::from_integer(0)
        );
        let debt_returns = debt_updates(
            asset.interest_accumulator,
            static_debt,
            U128::from_integer(amount),
            U128::from_integer(0)
        );

        assert(debt_returns.debt_value < debt_covered);

        storage.static_debt.at(owner).write(debt_returns.static_debt.to_integer());

        // @todo @LHerskind Need to support both private and public minting.
        let stable_coin = storage.stable_coin.read();
        let _ = Token::at(stable_coin).mint_public(to, amount).call(&mut context);
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
        let _ = Token::at(stable_coin).burn(from, amount, nonce).call(&mut context);
        let _ = Lending::at(context.this_address())._repay(AztecAddress::from_field(on_behalf_of), amount, stable_coin).enqueue(&mut context);
    }

    #[aztec(public)]
    fn repay_public(amount: Field, nonce: Field, owner: AztecAddress, stable_coin: AztecAddress) {
        let _ = Token::at(stable_coin).burn_public(context.msg_sender(), amount, nonce).call(&mut context);
        let _ = Lending::at(context.this_address())._repay(owner, amount, stable_coin).call(&mut context);
    }

    #[aztec(public)]
    #[aztec(internal)]
    fn _repay(owner: AztecAddress, amount: Field, stable_coin: AztecAddress) {
        let asset = Lending::at(context.this_address()).update_accumulator().call(&mut context);

        // To ensure that private is using the correct token.
        assert(stable_coin.eq(storage.stable_coin.read()));

        let static_debt = U128::from_integer(storage.static_debt.at(owner).read());
        let debt_returns = debt_updates(
            asset.interest_accumulator,
            static_debt,
            U128::from_integer(0),
            U128::from_integer(amount)
        );

        storage.static_debt.at(owner).write(debt_returns.static_debt.to_integer());
    }

    unconstrained fn get_asset(asset_id: Field) -> pub Asset {
        storage.assets.at(asset_id).read()
    }

    unconstrained fn get_position(owner: AztecAddress) -> pub Position {
        let collateral = storage.collateral.at(owner).read();
        let static_debt = storage.static_debt.at(owner).read();
        let asset: Asset = storage.assets.at(0).read();
        let debt = debt_value(
            U128::from_integer(static_debt),
            U128::from_integer(asset.interest_accumulator)
        ).to_integer();
        Position { collateral, static_debt, debt }
    }

    unconstrained fn get_assets() -> pub [AztecAddress; 2] {
        [storage.collateral_asset.read(), storage.stable_coin.read()]
    }
}
