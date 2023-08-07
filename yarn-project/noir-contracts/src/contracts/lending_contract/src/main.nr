mod storage;

contract Lending {
    use dep::aztec::{
        abi,
        abi::PrivateContextInputs,
        abi::PublicContextInputs
    };
    use dep::aztec::context::Context;
    use dep::aztec::oracle::{
        logs::emit_unencrypted_log,
        create_nullifier::create_nullifier,
    };
    use dep::aztec::public_call_stack_item::{
        PublicCallStackItem,
        call_public_function,
        call_public_function_no_args,
    };
    use crate::storage::{Storage, Tot, Account};

    // Constructs the contract.
    fn constructor(
        inputs: PrivateContextInputs
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        // Return private circuit public inputs. All private functions need to return this as it is part of the input of the private kernel.
        Context::new(inputs, 0).finish()
    }

    open fn init(
        inputs: PublicContextInputs
    ) -> pub Field {
        let storage = Storage::init();

        let asset = storage.assets.at(0);

        let tot = asset.read();

        assert (tot.last_updated_ts == 0);
        assert (tot.interest_accumulator == 0);

        asset.write(Tot {
            interest_accumulator: 1000000000,
            last_updated_ts: inputs.public_global_variables.timestamp as u120,
        });

        1
    }

    // Create a position.

    open fn update_tot(
        inputs: PublicContextInputs
    ) -> pub Field {
        let storage = Storage::init();

        let asset = storage.assets.at(0);
        let mut tot = asset.read();

        let dt: u120 = inputs.public_global_variables.timestamp as u120  - tot.last_updated_ts;

        // Need proper interest computation here.
        // Assume constant rate because too few reads :cry:
        let rate: u120 = 100000;
        // Just need something that increases, but :cry:
        let multiplier: u120 = 1000000000 + dt * rate;
        let divisor: u120 = 1000000000;

        tot.interest_accumulator = (tot.interest_accumulator * multiplier) / divisor;
        tot.last_updated_ts = inputs.public_global_variables.timestamp as u120;

        asset.write(tot);

        1
    }

    // This don't need to be on behalf of self. We should be able to repay on behalf of someone else.
    fn deposit_private(
        inputs: PrivateContextInputs,
        secret: Field,
        owner: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([
            secret,
            owner,
            amount
        ]));

        assert (!((secret == 0) as bool & (owner == 0) as bool));
        let mut account = owner;
        let me = inputs.call_context.msg_sender;

        if (secret != 0) {
            account = Account::new(me, secret).key();
        }

        // Unshield tokens into this contract.

        // _deposit(account, amount)
        let _callStackItem = context.call_public_function(
            inputs.call_context.storage_contract_address, 
            3009041984,
            [account, amount]
        );

        context.finish()
    }

    open fn deposit_public(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub Field {
        // @todo @LHerskind Transfer tokens into this contract. We can't do it now because too few writes.
    
        call_public_function(inputs.call_context.storage_contract_address, 3009041984, [owner, amount])[0]
    }

    open internal fn _deposit(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub Field {
        let _void = call_public_function_no_args(inputs.call_context.storage_contract_address, 1259373467)[0];

        let storage = Storage::init();

        let coll_loc = storage.collateral.at(owner);
        let collateral = coll_loc.read();

        coll_loc.write(collateral + amount);
        1
    }

    fn withdraw_private(
        inputs: PrivateContextInputs,
        secret: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([
            secret,
            amount
        ]));

        let me = inputs.call_context.msg_sender;
        let account = Account::new(me, secret).key();
        let _callStackItem = context.call_public_function(
            inputs.call_context.storage_contract_address, 
            1065861440,
            [account, amount]
        );

        context.finish()
    }


    open fn withdraw_public(
        inputs: PublicContextInputs,
        amount: Field
    ) -> pub Field {
        // @todo @LHerskind Transfer tokens into this contract. We can't do it now because too few writes.
    
        call_public_function(inputs.call_context.storage_contract_address, 1065861440, [inputs.call_context.msg_sender, amount])[0]
    }

     open internal fn _withdraw(
        inputs: PublicContextInputs,
        owner: Field,
        // recipient: Field,
        amount: Field
    ) -> pub Field {
        // Access control.
        let _void = call_public_function_no_args(inputs.call_context.storage_contract_address, 1259373467)[0];

        let storage = Storage::init();

        let coll_loc = storage.collateral.at(owner);
        let collateral: Field = coll_loc.read();

        let debt_loc = storage.static_debt.at(owner);
        let static_debt: Field = debt_loc.read();

        assert (collateral as u120 >= amount as u120);
        assert (collateral as u120 - amount as u120 >= static_debt as u120);

        coll_loc.write(collateral - amount);

        // Transfer tokens to recipient (could be with shielding or without, for now, can't to either).
        1
    }

    fn borrow_private(
        inputs: PrivateContextInputs,
        secret: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([
            secret,
            amount
        ]));

        let me = inputs.call_context.msg_sender;
        let account = Account::new(me, secret).key();

        let _callStackItem = context.call_public_function(
            inputs.call_context.storage_contract_address, 
            1462609836,
            [account, amount]
        );

        context.finish()
    }

    open fn borrow_public(
        inputs: PublicContextInputs,
        amount: Field
    ) -> pub Field {
        call_public_function(inputs.call_context.storage_contract_address, 1462609836, [inputs.call_context.msg_sender, amount])[0]
    }

    open internal fn _borrow(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub Field {
        // Access control.
        let _void = call_public_function_no_args(inputs.call_context.storage_contract_address, 1259373467)[0];

        let storage = Storage::init();

        let coll_loc = storage.collateral.at(owner);
        let collateral: Field = coll_loc.read();

        let debt_loc = storage.static_debt.at(owner);
        let static_debt: Field = debt_loc.read();

        assert (static_debt as u120 + amount as u120 < collateral as u120);

        debt_loc.write(static_debt + amount);

        // @todo @LHerskind Transferring funds to the recipient.

        1
    }

    // This don't need to be on behalf of self. We should be able to repay on behalf of someone else.
    fn repay_private(
        inputs: PrivateContextInputs,
        secret: Field,
        owner: Field,
        amount: Field
    ) -> distinct pub abi::PrivateCircuitPublicInputs {
        let mut context = Context::new(inputs, abi::hash_args([
            secret,
            owner,
            amount
        ]));

        assert (!((secret == 0) as bool & (owner == 0) as bool));
        let mut account = owner;
        let me = inputs.call_context.msg_sender;

        if (secret != 0) {
            account = Account::new(me, secret).key();
        }

        // @todo @lherskind Transfer tokens from me to this contract.

        let _callStackItem = context.call_public_function(
            inputs.call_context.storage_contract_address, 
            3985016136,
            [account, amount]
        );

        context.finish()
    }

    open fn repay_public(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub Field {
        // @todo @LHerskind Transfer tokens from msg.sender to this contract.
        call_public_function(inputs.call_context.storage_contract_address, 3985016136, [owner, amount])[0]
    }

    open internal fn _repay(
        inputs: PublicContextInputs,
        owner: Field,
        amount: Field
    ) -> pub Field {
        let _void = call_public_function_no_args(inputs.call_context.storage_contract_address, 1259373467)[0];

        let storage = Storage::init();
        // Should constrain access from private.
        // should only be callable from self from private.

        let debt_loc = storage.static_debt.at(owner);

        // @todo @LHerskind using "as" after using maps key => field have issues.
        // Need to be explicitly specified as field first.
        let static_debt: Field = debt_loc.read();

        assert (static_debt as u120 >= amount as u120);

        debt_loc.write(static_debt - amount);
        1
    }

    unconstrained fn getTot(
        assetId: Field,
    ) -> [Field; 2]{
        let storage = Storage::init();
        let asset = storage.assets.at(assetId);
        let tot = asset.read();
        [tot.interest_accumulator as Field, tot.last_updated_ts as Field]
    }

    unconstrained fn getPosition(
        owner: Field,
    ) -> [Field; 2] {
        let storage = Storage::init();
        let collateral = storage.collateral.at(owner).read();
        let debt = storage.static_debt.at(owner).read();
        [collateral, debt] 
    }
}
