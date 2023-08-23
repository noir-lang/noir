mod storage;

// Single asset lending contract. Just for show.
contract Lending {
    use dep::aztec::{
        abi,
        abi::PrivateContextInputs,
        abi::PublicContextInputs
    };
    use dep::aztec::context::{
        PrivateContext,
        PublicContext
    };
    use dep::aztec::oracle::{logs::emit_unencrypted_log};
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use crate::storage::{Storage, Tot, Account};

    struct Pos {
        collateral: Field,
        static_debt: Field,
    }

    // Constructs the contract.
    fn constructor(
        inputs: PrivateContextInputs
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        PrivateContext::new(inputs, 0).finish()
    }

    open fn init(
        inputs: PublicContextInputs
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([]));

        let storage = Storage::init();

        let asset = storage.assets.at(0);

        let tot = asset.read();

        assert (tot.last_updated_ts == 0);
        assert (tot.interest_accumulator == 0);

        asset.write(Tot {
            interest_accumulator: 1000000000,
            last_updated_ts: context.timestamp() as u120,
        });

        context.return_values.push(1);
        context.finish()
    }

    // Create a position.

    open fn update_tot(
        inputs: PublicContextInputs
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([]));
        let storage = Storage::init();

        let asset = storage.assets.at(0);
        let mut tot = asset.read();

        let dt: u120 = context.timestamp() as u120  - tot.last_updated_ts;

        // Need proper interest computation here.
        // Assume constant rate because too few reads :cry:
        let rate: u120 = 100000;
        // Just need something that increases, but :cry:
        let multiplier: u120 = 1000000000 + dt * rate;
        let divisor: u120 = 1000000000;

        tot.interest_accumulator = (tot.interest_accumulator * multiplier) / divisor;
        tot.last_updated_ts = context.timestamp() as u120;

        asset.write(tot);

        context.return_values.push(1);
        context.finish()
    }

    // This don't need to be on behalf of self. We should be able to repay on behalf of someone else.
    fn deposit_private(
        inputs: PrivateContextInputs,
        secret: Field,
        owner: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            secret,
            owner,
            amount
        ]));

        assert (!((secret == 0) as bool & (owner == 0) as bool));
        let mut account = owner;
        let me = context.msg_sender();

        if (secret != 0) {
            account = Account::new(me, secret).key();
        }

        // Unshield tokens into this contract.

        // _deposit(account, amount)
        let _callStackItem = context.call_public_function(
            context.this_address(), 
            3009041984,
            [account, amount]
        );

        context.finish()
    }

    open fn deposit_public(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([owner, amount]));
        // @todo @LHerskind Transfer tokens into this contract. We can't do it now because too few writes.
    
        let return_values = context.call_public_function(context.this_address(), 3009041984, [owner, amount]);

        context.return_values.push(return_values[0]);
        context.finish()
    }

    open internal fn _deposit(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([owner, amount]));

        let _void = context.call_public_function_no_args(context.this_address(), 1259373467)[0];

        let storage = Storage::init();

        let coll_loc = storage.collateral.at(owner);
        let collateral = coll_loc.read();

        coll_loc.write(collateral + amount);

        context.return_values.push(1);
        context.finish()
    }

    fn withdraw_private(
        inputs: PrivateContextInputs,
        secret: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            secret,
            amount
        ]));

        let me = context.msg_sender();
        let account = Account::new(me, secret).key();
        let _callStackItem = context.call_public_function(
            context.this_address(), 
            1065861440,
            [account, amount]
        );

        context.finish()
    }


    open fn withdraw_public(
        inputs: PublicContextInputs,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([amount]));
        // @todo @LHerskind Transfer tokens into this contract. We can't do it now because too few writes.
    
        let return_values = context.call_public_function(context.this_address(), 1065861440, [context.msg_sender(), amount]);

        context.return_values.push(return_values[0]);
        context.finish()
    }

     open internal fn _withdraw(
        inputs: PublicContextInputs,
        owner: Field,
        // recipient: Field,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        // Access control.
        let mut context = PublicContext::new(inputs, abi::hash_args([owner, amount]));

        let _void = context.call_public_function_no_args(context.this_address(), 1259373467)[0];

        let storage = Storage::init();

        let coll_loc = storage.collateral.at(owner);
        let collateral: Field = coll_loc.read();

        let debt_loc = storage.static_debt.at(owner);
        let static_debt: Field = debt_loc.read();

        assert (collateral as u120 >= amount as u120);
        assert (collateral as u120 - amount as u120 >= static_debt as u120);

        coll_loc.write(collateral - amount);

        // Transfer tokens to recipient (could be with shielding or without, for now, can't to either).
        context.return_values.push(1);
        context.finish()
    }

    fn borrow_private(
        inputs: PrivateContextInputs,
        secret: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            secret,
            amount
        ]));

        let me = context.msg_sender();
        let account = Account::new(me, secret).key();

        let _callStackItem = context.call_public_function(
            context.this_address(), 
            1462609836,
            [account, amount]
        );

        context.finish()
    }

    open fn borrow_public(
        inputs: PublicContextInputs,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([amount]));
        let return_values = context.call_public_function(context.this_address(), 1462609836, [context.msg_sender(), amount]);

        context.return_values.push(return_values[0]);
        context.finish()
    }

    open internal fn _borrow(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([owner, amount]));
        // Access control.
        let _void = context.call_public_function_no_args(context.this_address(), 1259373467)[0];

        let storage = Storage::init();

        let coll_loc = storage.collateral.at(owner);
        let collateral: Field = coll_loc.read();

        let debt_loc = storage.static_debt.at(owner);
        let static_debt: Field = debt_loc.read();

        assert (static_debt as u120 + amount as u120 < collateral as u120);

        debt_loc.write(static_debt + amount);

        // @todo @LHerskind Transferring funds to the recipient.

        context.return_values.push(1);
        context.finish()
    }

    // This don't need to be on behalf of self. We should be able to repay on behalf of someone else.
    fn repay_private(
        inputs: PrivateContextInputs,
        secret: Field,
        owner: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = PrivateContext::new(inputs, abi::hash_args([
            secret,
            owner,
            amount
        ]));

        assert (!((secret == 0) as bool & (owner == 0) as bool));
        let mut account = owner;
        let me = context.msg_sender();

        if (secret != 0) {
            account = Account::new(me, secret).key();
        }

        // @todo @lherskind Transfer tokens from me to this contract.

        let _callStackItem = context.call_public_function(
            context.this_address(), 
            3985016136,
            [account, amount]
        );

        context.finish()
    }

    open fn repay_public(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([owner, amount]));
        // @todo @LHerskind Transfer tokens from msg.sender to this contract.
        let return_values = context.call_public_function(context.this_address(), 3985016136, [owner, amount]);

        context.return_values.push(return_values[0]);
        context.finish()
    }

    open internal fn _repay(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub abi::PublicCircuitPublicInputs {
        let mut context = PublicContext::new(inputs, abi::hash_args([owner, amount]));
        let _void = context.call_public_function_no_args(context.this_address(), 1259373467)[0];

        let storage = Storage::init();
        // Should constrain access from private.
        // should only be callable from self from private.

        let debt_loc = storage.static_debt.at(owner);

        // @todo @LHerskind using "as" after using maps key => field have issues.
        // Need to be explicitly specified as field first.
        let static_debt: Field = debt_loc.read();

        assert (static_debt as u120 >= amount as u120);

        debt_loc.write(static_debt - amount);
        
        context.return_values.push(1);
        context.finish()
    }

    unconstrained fn getTot(
        assetId: Field,
    ) -> Tot {
        let storage = Storage::init();
        storage.assets.at(assetId).read()
    }

    unconstrained fn getPosition(
        owner: Field,
    ) -> Pos {
        let storage = Storage::init();
        let collateral = storage.collateral.at(owner).read();
        let static_debt = storage.static_debt.at(owner).read();
        Pos {collateral, static_debt}
    }
}
