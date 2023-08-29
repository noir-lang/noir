mod storage;
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
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use crate::storage::{Storage, Asset};
    use dep::safe_math::SafeU120;
    use crate::interest_math::compute_multiplier;
    use crate::helpers::{covered_by_collateral, DebtReturn, debt_updates, debt_value, compute_identifier};
    use crate::interfaces::{Token, Lending, PriceFeed};

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
    open fn init(
        oracle_address: Field,
        loan_to_value: Field,
        collateral_asset: Field,
        stable_coin: Field,
    ) {
        let storage = Storage::init();
        let asset_loc = storage.assets.at(0);
        let asset = asset_loc.read();

        assert (loan_to_value as u120 <= 10000);
        assert (asset.last_updated_ts == 0);
        assert (asset.interest_accumulator == 0);

        asset_loc.write(Asset {
            interest_accumulator: 1000000000,
            last_updated_ts: context.timestamp() as u120,
            loan_to_value: loan_to_value as u120,
            oracle_address,
        });

        storage.collateral_asset.write(collateral_asset);
        storage.stable_coin.write(stable_coin);

        context.return_values.push(1);
    }

    // Create a position.
    // keccak256("update_accumulator()") >> 224 -> 0x1873b536
    #[aztec(public)]
    open fn update_accumulator() {
        let storage = Storage::init();

        let asset_loc = storage.assets.at(0);
        let mut asset = asset_loc.read();

        let dt: SafeU120 = SafeU120{value: context.timestamp() as u120}.sub(SafeU120{value: asset.last_updated_ts});

        // Only update if time has passed.
        if (!dt.is_zero()) {
            let precision: SafeU120 = SafeU120{value: 1000000000};
            let rate_per_second: u120 = 1268391679; // 4% yearly rate / (60 * 60 * 24 * 365)
            // if rate_per_second < sqrt(WAD) our approx is eq precision + rate * dt
            let multiplier = compute_multiplier(rate_per_second, dt);

            // accumulator *= multiplier, and multiplier >= 1
            asset.interest_accumulator = SafeU120{value: asset.interest_accumulator}.mul_div(multiplier, precision).value;
            asset.last_updated_ts = context.timestamp() as u120;

            asset_loc.write(asset);
        }

        context.return_values.push_array(asset.serialise());
    }

    // This don't need to be on behalf of self. We should be able to repay on behalf of someone else.
    #[aztec(private)]
    fn deposit_private(
        secret: Field,
        asset_owner: Field,
        on_behalf_of: Field,
        amount: Field,
        collateral_asset: Field,
    ) {
        let on_behalf_of = compute_identifier(secret, on_behalf_of, context.msg_sender());
        let _res = Token::at(collateral_asset).unshield(&mut context, asset_owner, context.this_address(), amount);
        // _deposit(on_behalf_of, amount, collateral_asset)
        let _callStackItem2 = context.call_public_function(context.this_address(), 0x08506e50,[on_behalf_of, amount, collateral_asset]);
    }

    #[aztec(public)]
    open fn deposit_public(
        owner: Field,
        amount: Field,
        collateral_asset: Field,
    ) {
        Token::at(collateral_asset).transfer_from_pub(context, context.msg_sender(), context.this_address(), amount);
        let return_values = context.call_public_function(context.this_address(), 0x08506e50, [owner, amount, collateral_asset]);
        context.return_values.push(return_values[0]);
    }

    #[aztec(public)]
    // keccak256("_deposit(field,field,field)") >> 224 -> 0x08506e50
    open internal fn _deposit(
        owner: Field,
        amount: Field,
        collateral_asset: Field,
    ) {
        let _asset = Lending::at(context.this_address()).update_accumulator(context);
        let storage = Storage::init();

        let coll_asset = storage.collateral_asset.read();
        assert(coll_asset == collateral_asset);

        let coll_loc = storage.collateral.at(owner);
        let collateral = coll_loc.read();
        coll_loc.write(collateral + amount);

        context.return_values.push(1);
    }

    #[aztec(private)]
    fn withdraw_private(
        secret: Field,
        to: Field,
        amount: Field
    ) {
        let on_behalf_of = compute_identifier(secret, 0, context.msg_sender());
        let _callStackItem = context.call_public_function(context.this_address(), 0x5af6f634, [on_behalf_of, to, amount]);
    }

    #[aztec(public)]
    open fn withdraw_public(
        to: Field,
        amount: Field,
    ) {
        // _withdraw(msg.sender, to, amount);
        let return_values = context.call_public_function(context.this_address(), 0x5af6f634, [context.msg_sender(), to, amount]);
        context.return_values.push(return_values[0]);
    }

    // keccak256("_withdraw(field,field,field)") >> 224 -> 0x5af6f634
    #[aztec(public)]
    open internal fn _withdraw(
        owner: Field,
        recipient: Field,
        amount: Field
    ) {
        let asset = Lending::at(context.this_address()).update_accumulator(context);
        let price = PriceFeed::at(asset.oracle_address).get_price(context);
        
        let storage = Storage::init();

        let coll_loc = storage.collateral.at(owner);
        let collateral: Field = coll_loc.read();

        let debt_loc = storage.static_debt.at(owner);
        let static_debt: Field = debt_loc.read();

        // debt_covered will revert if decrease would leave insufficient collateral to cover debt.
        // or trying to remove more collateral than available
        let debt_covered = covered_by_collateral(price, asset.loan_to_value, collateral as u120, 0, amount as u120);
        let debt_returns = debt_updates(asset.interest_accumulator, static_debt as u120, 0, 0);

        assert (debt_returns.debt_value < debt_covered);

        coll_loc.write(collateral - amount);

        // @todo @LHerskind Support both shielding and transfers (for now just transfer)
        let collateral_asset = storage.collateral_asset.read();
        Token::at(collateral_asset).transfer_pub(context, recipient, amount);

        context.return_values.push(1);
    }

    #[aztec(private)]
    fn borrow_private(
        secret: Field,
        to: Field,
        amount: Field
    ) {
        let on_behalf_of = compute_identifier(secret, 0, context.msg_sender());
        // _borrow(msg.sender, to, amount)
        let _callStackItem = context.call_public_function(context.this_address(), 0xceffa31a, [on_behalf_of, to, amount]);
    }

    #[aztec(public)]
    open fn borrow_public(
        to: Field,
        amount: Field
    ) {
        // _borrow(msg.sender, to, amount)
        let return_values = context.call_public_function(context.this_address(), 0xceffa31a, [context.msg_sender(), to, amount]);
        context.return_values.push(return_values[0]);
    }

    // keccak256("_borrow(field,field,field)") >> 224 -> 0xceffa31a
    #[aztec(public)]
    open internal fn _borrow(
        owner: Field,
        to: Field,
        amount: Field
    ) {
        let asset = Lending::at(context.this_address()).update_accumulator(context);
        let price = PriceFeed::at(asset.oracle_address).get_price(context);
        
        let storage = Storage::init();

        // Fetch collateral and static_debt, compute health of current position
        let collateral = storage.collateral.at(owner).read() as u120;
        let static_debt = storage.static_debt.at(owner).read() as u120;

        let debt_covered = covered_by_collateral(price, asset.loan_to_value, collateral, 0, 0);
        let debt_returns = debt_updates(asset.interest_accumulator, static_debt, amount as u120, 0);

        assert (debt_returns.debt_value < debt_covered);

        storage.static_debt.at(owner).write(debt_returns.static_debt as Field);

        // @todo @LHerskind Need to support both private and public minting.
        let stable_coin = storage.stable_coin.read();
        Token::at(stable_coin).owner_mint_pub(context, to, amount);

        context.return_values.push(1);
    }

    #[aztec(private)]
    fn repay_private(
        secret: Field,
        asset_owner: Field,
        on_behalf_of: Field,
        amount: Field,
        stable_coin: Field,
    ) {
        let on_behalf_of = compute_identifier(secret, on_behalf_of, context.msg_sender());
        let _res = Token::at(stable_coin).unshield(&mut context, asset_owner, context.this_address(), amount);
        let _callStackItem = context.call_public_function(context.this_address(), 0xfa94ab54, [on_behalf_of, amount, stable_coin]);
    }

    #[aztec(public)]
    open fn repay_public(
        owner: Field,
        amount: Field,
        stable_coin: Field,
    ) {
        // Should probably just burn the tokens actually :thinking:
        Token::at(stable_coin).transfer_from_pub(context, context.msg_sender(), context.this_address(), amount);
        let return_values = context.call_public_function(context.this_address(), 0xfa94ab54, [owner, amount, stable_coin]);
        context.return_values.push(return_values[0]);
    }

    // keccak256("_repay(field,field,field)") >> 224 -> 0xfa94ab54
    #[aztec(public)]
    open internal fn _repay(
        owner: Field,
        amount: Field,
        stable_coin: Field,
    ) {
        let asset = Lending::at(context.this_address()).update_accumulator(context);
        let storage = Storage::init();

        // To ensure that private is using the correct token.
        assert(stable_coin == storage.stable_coin.read());

        let static_debt = storage.static_debt.at(owner).read() as u120;
        let debt_returns = debt_updates(asset.interest_accumulator, static_debt, 0, amount as u120);

        storage.static_debt.at(owner).write(debt_returns.static_debt as Field);
        
        context.return_values.push(1);
    }

    unconstrained fn get_asset(
        assetId: Field,
    ) -> Asset {
        let storage = Storage::init();
        storage.assets.at(assetId).read()
    }

    unconstrained fn get_position(
        owner: Field,
    ) -> Position {
        let storage = Storage::init();
        let collateral = storage.collateral.at(owner).read();
        let static_debt = storage.static_debt.at(owner).read();
        let asset = storage.assets.at(0).read();
        let debt = debt_value(static_debt as u120, asset.interest_accumulator as u120) as Field;
        Position {collateral, static_debt, debt}
    }

    unconstrained fn get_assets() -> [Field; 2] {
        let storage = Storage::init();
        [storage.collateral_asset.read(), storage.stable_coin.read()]
    }
}
