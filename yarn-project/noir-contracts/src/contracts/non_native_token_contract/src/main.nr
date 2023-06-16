contract NonNativeToken {
    mod balance_utils;
    use dep::aztec3::abi::PublicContextInputs;
    use dep::aztec3::abi::PrivateContextInputs;

    use dep::aztec3::context::PrivateFunctionContext;
    use dep::aztec3::types::point::Point;
    
    // Private state
    use dep::aztec3::state_vars::map::Map;
    use dep::aztec3::state_vars::set::Set;
    use dep::aztec3::notes::value_note::Note;

    // Public state
    use dep::aztec3::state_vars::storage_map::StorageMap;
    use dep::aztec3::state_vars::storage_field::StorageField;

    // oracles
    use dep::aztec3::oracle::logs::emit_encrypted_log;
    use dep::aztec3::oracle::create_l2_to_l1_message::create_l2_to_l1_message;

    // public messaging
    use dep::aztec3::messaging::consume_l1_to_l2_message_public;


    //*********************************/
    global balances = Map { storage_slot: 1 };
    global public_balances = StorageMap { storage_slot: 2 };
    // Should eventually be replaced by:
    // `balances: Mapping<Field, Set<Note>>;`
    //*********************************/

    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: pub PrivateContextInputs,
        //*********************************/
        initial_supply: pub Field, 
        owner: pub Point,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut context = PrivateFunctionContext::new();
        context.args = context.args.push_array([initial_supply, owner.x, owner.y]);

        let owner_balance = balances.at(owner.x);
        let note = Note::new(initial_supply, owner);

        // Insert note and emit encrypted note preimage via oracle call
        context = owner_balance.insert(context, note);
        constrain emit_encrypted_log(inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note) == 0;

        context.finish(inputs)
    }

    // Mint Private Function
    // This mint function differs to the typical token mint function as it only allows minting 
    // upon consuming valid messages from a token portal contract
    fn mint(
        //*********************************/
        // Should eventually be hidden:
        inputs: pub PrivateContextInputs,
        //*********************************/
        amount: pub Field, 
        owner: pub Point,
        owner_address: pub Field,
        // This field should be hidden
        msg_key: pub Field,
        secret: pub Field,
        canceller: pub Field,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut context = PrivateFunctionContext::new();
        context.args = context.args.push_array([amount, owner.x, owner.y, owner_address, msg_key, secret]);

        let content_hash = _get_mint_content_hash(amount, owner_address, canceller);

        // Get the l1 message from an oracle call
        let updated_context = context.consume_l1_to_l2_message(inputs, msg_key, content_hash, secret);
        context = updated_context;

        let owner_balance = balances.at(owner.x);
        let note = Note::new(amount, owner);

        // Insert note and emit encrypted note preimage via oracle call
        context = owner_balance.insert(context, note);
        constrain emit_encrypted_log(inputs.call_context.storage_contract_address, owner_balance.storage_slot, note.owner, note) == 0;

        context.finish(inputs)
    }



    fn withdraw(
        //*********************************/
        // Should eventually be hidden:
        inputs: pub PrivateContextInputs,
        //*********************************/
        amount: pub Field, 
        sender: pub Point, 
        recipient: pub Field, // ethereum address in the field
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut initialContext = PrivateFunctionContext::new();
        initialContext.args = initialContext.args.push_array([amount, sender.x, sender.y, recipient]);

        let sender_balance = balances.at(sender.x);
        let (mut context, (note1, note2)) = sender_balance.get_2(initialContext);

        // TODO: Removing this return breaks the circuit (???)
        context.return_values = context.return_values.push(note1.is_real as Field);
        context.return_values = context.return_values.push(note2.is_real as Field);

        note1.validate(sender);
        note2.validate(sender);

        let sum = note1.value + note2.value;
        constrain sum as u64 >= amount as u64;
        let change_value = sum - amount;

        context = sender_balance.remove(inputs, context, note1);
        context = sender_balance.remove(inputs, context, note2);

        let change_note = Note::new(change_value, sender);

        // Insert note and emit encrypted note preimage via oracle call
        context = sender_balance.insert(context, change_note);
        constrain emit_encrypted_log(inputs.call_context.storage_contract_address, sender_balance.storage_slot, change_note.owner, change_note) == 0;
        
        let content = _get_withdraw_content_hash(amount, recipient);

        context = context.message_portal(content);

        context.finish(inputs)
    }

    // Mint Public Function
    // This mint function differs to the typical token mint function as it only allows minting 
    // upon consuming valid messages from a token portal contract
    open fn mintPublic(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        amount: Field, 
        owner_address: Field,
        // This field should be hidden
        msg_key: Field,
        secret: Field,
        canceller: Field,
        _padding: [Field; dep::aztec3::abi::MAX_ARGS - 5]
    ) -> Field {
        let content_hash = _get_mint_content_hash(amount, owner_address, canceller);

        // Consume message and emit nullifier
        consume_l1_to_l2_message_public(inputs, msg_key, content_hash, secret);

        // Update the public balance
        let recipient_balance = public_balances.at(owner_address);
        recipient_balance.write(recipient_balance.read() + amount)
    }


    // Withdraw Public Function
    open fn withdrawPublic(
        //*********************************/
        // Should eventually be hidden:
        inputs: PublicContextInputs,
        //*********************************/
        amount: Field,
        recipient: Field,
        _padding: [Field; dep::aztec3::abi::MAX_ARGS - 2]
    ) {

        let sender = inputs.call_context.msg_sender;
        let sender_balance = public_balances.at(sender);

        let current_sender_balance = sender_balance.read();

        if (current_sender_balance as u120) > (amount as u120) {
            let _void1 = sender_balance.write(current_sender_balance - amount);
        }  
        // TODO: Revert if there is not enough balance

        let content = _get_withdraw_content_hash(amount, recipient);

        // Emit the l2 to l1 message
        create_l2_to_l1_message(content);
    }


    fn transfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: pub PrivateContextInputs,
        //*********************************/
        amount: pub Field, 
        sender: pub Point, 
        recipient: pub Point,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        let mut initialContext = PrivateFunctionContext::new();
        initialContext.args = initialContext.args.push_array([amount, sender.x, sender.y, recipient.x, recipient.y]);

        let sender_balance = balances.at(sender.x);
        let (mut context, (note1, note2)) = sender_balance.get_2(initialContext);

        // TODO: Removing this return breaks the circuit (???)
        context.return_values = context.return_values.push(note1.is_real as Field);
        context.return_values = context.return_values.push(note2.is_real as Field);

        note1.validate(sender);
        note2.validate(sender);

        let sum = note1.value + note2.value;
        constrain sum as u64 >= amount as u64;
        let change_value = sum - amount;

        context = sender_balance.remove(inputs, context, note1);
        context = sender_balance.remove(inputs, context, note2);

        let change_note = Note::new(change_value, sender);
        let recipient_note = Note::new(amount, recipient);

        let recipient_balance = balances.at(recipient.x);

        context = recipient_balance.insert(context, recipient_note);
        context = sender_balance.insert(context, change_note);

        constrain emit_encrypted_log(inputs.call_context.storage_contract_address, recipient_balance.storage_slot, recipient_note.owner, recipient_note) == 0;
        constrain emit_encrypted_log(inputs.call_context.storage_contract_address, sender_balance.storage_slot, change_note.owner, change_note) == 0;

        context.finish(inputs)
    } 

    /// ABI getBalance type "unconstrained"
    fn getBalance(
        owner: Point,
        _padding: [Field; dep::aztec3::abi::MAX_ARGS - 2]
    ) -> pub Field {
        let owner_balance = balances.at(owner.x);
        
        balance_utils::get_balance(owner_balance.storage_slot)
    }


    fn _get_mint_content_hash(amount: Field, owner_address: Field, canceller: Field) -> pub Field {
        let mut hash_bytes: [u8; 100] = [0; 100];
        let amount_bytes = amount.to_be_bytes(32);
        let recipient_bytes = owner_address.to_be_bytes(32);
        let canceller_bytes = canceller.to_be_bytes(32);

        // Once loops are performant this will be moved into l1 message
        // Unroll loops because otherwise takes forever to compile
        // for i in range(32):
        //     print('hash_bytes[{0}] = amount_bytes[{1}];'.format(i, i))
        // for i in range(32):
        //     print('hash_bytes[{0} + 32] = recipient_bytes[{1}];'.format(i, i))
        // for i in range(32):
        //     print('hash_bytes[{0} + 64] = canceller_bytes[{1}];'.format(i, i))
        
        // Function selector: 0xeeb73071 keccak256('mint(uint256,bytes32,address)')
        hash_bytes[0] = 0xee;
        hash_bytes[1] = 0xb7;
        hash_bytes[2] = 0x30;
        hash_bytes[3] = 0x71;

        hash_bytes[0 + 4] = amount_bytes[0];
        hash_bytes[1 + 4] = amount_bytes[1];
        hash_bytes[2 + 4] = amount_bytes[2];
        hash_bytes[3 + 4] = amount_bytes[3];
        hash_bytes[4 + 4] = amount_bytes[4];
        hash_bytes[5 + 4] = amount_bytes[5];
        hash_bytes[6 + 4] = amount_bytes[6];
        hash_bytes[7 + 4] = amount_bytes[7];
        hash_bytes[8 + 4] = amount_bytes[8];
        hash_bytes[9 + 4] = amount_bytes[9];
        hash_bytes[10 + 4] = amount_bytes[10];
        hash_bytes[11 + 4] = amount_bytes[11];
        hash_bytes[12 + 4] = amount_bytes[12];
        hash_bytes[13 + 4] = amount_bytes[13];
        hash_bytes[14 + 4] = amount_bytes[14];
        hash_bytes[15 + 4] = amount_bytes[15];
        hash_bytes[16 + 4] = amount_bytes[16];
        hash_bytes[17 + 4] = amount_bytes[17];
        hash_bytes[18 + 4] = amount_bytes[18];
        hash_bytes[19 + 4] = amount_bytes[19];
        hash_bytes[20 + 4] = amount_bytes[20];
        hash_bytes[21 + 4] = amount_bytes[21];
        hash_bytes[22 + 4] = amount_bytes[22];
        hash_bytes[23 + 4] = amount_bytes[23];
        hash_bytes[24 + 4] = amount_bytes[24];
        hash_bytes[25 + 4] = amount_bytes[25];
        hash_bytes[26 + 4] = amount_bytes[26];
        hash_bytes[27 + 4] = amount_bytes[27];
        hash_bytes[28 + 4] = amount_bytes[28];
        hash_bytes[29 + 4] = amount_bytes[29];
        hash_bytes[30 + 4] = amount_bytes[30];
        hash_bytes[31 + 4] = amount_bytes[31];

        hash_bytes[0 + 36] = recipient_bytes[0];
        hash_bytes[1 + 36] = recipient_bytes[1];
        hash_bytes[2 + 36] = recipient_bytes[2];
        hash_bytes[3 + 36] = recipient_bytes[3];
        hash_bytes[4 + 36] = recipient_bytes[4];
        hash_bytes[5 + 36] = recipient_bytes[5];
        hash_bytes[6 + 36] = recipient_bytes[6];
        hash_bytes[7 + 36] = recipient_bytes[7];
        hash_bytes[8 + 36] = recipient_bytes[8];
        hash_bytes[9 + 36] = recipient_bytes[9];
        hash_bytes[10 + 36] = recipient_bytes[10];
        hash_bytes[11 + 36] = recipient_bytes[11];
        hash_bytes[12 + 36] = recipient_bytes[12];
        hash_bytes[13 + 36] = recipient_bytes[13];
        hash_bytes[14 + 36] = recipient_bytes[14];
        hash_bytes[15 + 36] = recipient_bytes[15];
        hash_bytes[16 + 36] = recipient_bytes[16];
        hash_bytes[17 + 36] = recipient_bytes[17];
        hash_bytes[18 + 36] = recipient_bytes[18];
        hash_bytes[19 + 36] = recipient_bytes[19];
        hash_bytes[20 + 36] = recipient_bytes[20];
        hash_bytes[21 + 36] = recipient_bytes[21];
        hash_bytes[22 + 36] = recipient_bytes[22];
        hash_bytes[23 + 36] = recipient_bytes[23];
        hash_bytes[24 + 36] = recipient_bytes[24];
        hash_bytes[25 + 36] = recipient_bytes[25];
        hash_bytes[26 + 36] = recipient_bytes[26];
        hash_bytes[27 + 36] = recipient_bytes[27];
        hash_bytes[28 + 36] = recipient_bytes[28];
        hash_bytes[29 + 36] = recipient_bytes[29];
        hash_bytes[30 + 36] = recipient_bytes[30];
        hash_bytes[31 + 36] = recipient_bytes[31];

        hash_bytes[0 + 68] = canceller_bytes[0];
        hash_bytes[1 + 68] = canceller_bytes[1];
        hash_bytes[2 + 68] = canceller_bytes[2];
        hash_bytes[3 + 68] = canceller_bytes[3];
        hash_bytes[4 + 68] = canceller_bytes[4];
        hash_bytes[5 + 68] = canceller_bytes[5];
        hash_bytes[6 + 68] = canceller_bytes[6];
        hash_bytes[7 + 68] = canceller_bytes[7];
        hash_bytes[8 + 68] = canceller_bytes[8];
        hash_bytes[9 + 68] = canceller_bytes[9];
        hash_bytes[10 + 68] = canceller_bytes[10];
        hash_bytes[11 + 68] = canceller_bytes[11];
        hash_bytes[12 + 68] = canceller_bytes[12];
        hash_bytes[13 + 68] = canceller_bytes[13];
        hash_bytes[14 + 68] = canceller_bytes[14];
        hash_bytes[15 + 68] = canceller_bytes[15];
        hash_bytes[16 + 68] = canceller_bytes[16];
        hash_bytes[17 + 68] = canceller_bytes[17];
        hash_bytes[18 + 68] = canceller_bytes[18];
        hash_bytes[19 + 68] = canceller_bytes[19];
        hash_bytes[20 + 68] = canceller_bytes[20];
        hash_bytes[21 + 68] = canceller_bytes[21];
        hash_bytes[22 + 68] = canceller_bytes[22];
        hash_bytes[23 + 68] = canceller_bytes[23];
        hash_bytes[24 + 68] = canceller_bytes[24];
        hash_bytes[25 + 68] = canceller_bytes[25];
        hash_bytes[26 + 68] = canceller_bytes[26];
        hash_bytes[27 + 68] = canceller_bytes[27];
        hash_bytes[28 + 68] = canceller_bytes[28];
        hash_bytes[29 + 68] = canceller_bytes[29];
        hash_bytes[30 + 68] = canceller_bytes[30];
        hash_bytes[31 + 68] = canceller_bytes[31];
        
        let content_sha256 = dep::std::hash::sha256(hash_bytes);

        // // Convert the content_sha256 to a field element
        let mut v = 1;
        let mut high = 0 as Field;
        let mut low = 0 as Field;

        // Unroll loops because otherwise takes forever to compile
        // for i in [15 - i for i in range(16)]:
        //     print('high = high + (content_sha256[{0}] as Field) * v;'.format(i))
        //     print('low = low + (content_sha256[16 + {0}] as Field) * v;'.format(i))
        //     print('v = v * 256;');
        high = high + (content_sha256[15] as Field) * v;
        low = low + (content_sha256[16 + 15] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[14] as Field) * v;
        low = low + (content_sha256[16 + 14] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[13] as Field) * v;
        low = low + (content_sha256[16 + 13] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[12] as Field) * v;
        low = low + (content_sha256[16 + 12] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[11] as Field) * v;
        low = low + (content_sha256[16 + 11] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[10] as Field) * v;
        low = low + (content_sha256[16 + 10] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[9] as Field) * v;
        low = low + (content_sha256[16 + 9] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[8] as Field) * v;
        low = low + (content_sha256[16 + 8] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[7] as Field) * v;
        low = low + (content_sha256[16 + 7] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[6] as Field) * v;
        low = low + (content_sha256[16 + 6] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[5] as Field) * v;
        low = low + (content_sha256[16 + 5] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[4] as Field) * v;
        low = low + (content_sha256[16 + 4] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[3] as Field) * v;
        low = low + (content_sha256[16 + 3] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[2] as Field) * v;
        low = low + (content_sha256[16 + 2] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[1] as Field) * v;
        low = low + (content_sha256[16 + 1] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[0] as Field) * v;
        low = low + (content_sha256[16 + 0] as Field) * v;
        v = v * 256;

        // Abuse that a % p + b % p = (a + b) % p and that low < p
        let content_hash = low + high * v;
        content_hash
    }

    fn _get_withdraw_content_hash(amount: Field, recipient: Field) -> pub Field {
        // Compute the content hash
        // Compute sha256(selector || amount || recipient)
        // then convert to a single field element
        // add that to the l2 to l1 messages
        let mut hash_bytes: [u8; 68] = [0; 68];
        let amount_bytes = amount.to_be_bytes(32);
        let recipient_bytes = recipient.to_be_bytes(32);

        // 0x00f714ce, selector for "withdraw(uint256,address)"
        hash_bytes[0] = 0x00;
        hash_bytes[1] = 0xf7;
        hash_bytes[2] = 0x14;
        hash_bytes[3] = 0xce;

        // Unroll loops because otherwise takes forever to compile
        // for i in range(32):
        //     print('hash_bytes[{0}] = amount_bytes[{1}];'.format(i+4, i))
        // for i in range(32):
        //     print('hash_bytes[{0} + 32] = recipient_bytes[{1}];'.format(i+4, i))
        hash_bytes[4] = amount_bytes[0];
        hash_bytes[5] = amount_bytes[1];
        hash_bytes[6] = amount_bytes[2];
        hash_bytes[7] = amount_bytes[3];
        hash_bytes[8] = amount_bytes[4];
        hash_bytes[9] = amount_bytes[5];
        hash_bytes[10] = amount_bytes[6];
        hash_bytes[11] = amount_bytes[7];
        hash_bytes[12] = amount_bytes[8];
        hash_bytes[13] = amount_bytes[9];
        hash_bytes[14] = amount_bytes[10];
        hash_bytes[15] = amount_bytes[11];
        hash_bytes[16] = amount_bytes[12];
        hash_bytes[17] = amount_bytes[13];
        hash_bytes[18] = amount_bytes[14];
        hash_bytes[19] = amount_bytes[15];
        hash_bytes[20] = amount_bytes[16];
        hash_bytes[21] = amount_bytes[17];
        hash_bytes[22] = amount_bytes[18];
        hash_bytes[23] = amount_bytes[19];
        hash_bytes[24] = amount_bytes[20];
        hash_bytes[25] = amount_bytes[21];
        hash_bytes[26] = amount_bytes[22];
        hash_bytes[27] = amount_bytes[23];
        hash_bytes[28] = amount_bytes[24];
        hash_bytes[29] = amount_bytes[25];
        hash_bytes[30] = amount_bytes[26];
        hash_bytes[31] = amount_bytes[27];
        hash_bytes[32] = amount_bytes[28];
        hash_bytes[33] = amount_bytes[29];
        hash_bytes[34] = amount_bytes[30];
        hash_bytes[35] = amount_bytes[31];

        hash_bytes[4 + 32] = recipient_bytes[0];
        hash_bytes[5 + 32] = recipient_bytes[1];
        hash_bytes[6 + 32] = recipient_bytes[2];
        hash_bytes[7 + 32] = recipient_bytes[3];
        hash_bytes[8 + 32] = recipient_bytes[4];
        hash_bytes[9 + 32] = recipient_bytes[5];
        hash_bytes[10 + 32] = recipient_bytes[6];
        hash_bytes[11 + 32] = recipient_bytes[7];
        hash_bytes[12 + 32] = recipient_bytes[8];
        hash_bytes[13 + 32] = recipient_bytes[9];
        hash_bytes[14 + 32] = recipient_bytes[10];
        hash_bytes[15 + 32] = recipient_bytes[11];
        hash_bytes[16 + 32] = recipient_bytes[12];
        hash_bytes[17 + 32] = recipient_bytes[13];
        hash_bytes[18 + 32] = recipient_bytes[14];
        hash_bytes[19 + 32] = recipient_bytes[15];
        hash_bytes[20 + 32] = recipient_bytes[16];
        hash_bytes[21 + 32] = recipient_bytes[17];
        hash_bytes[22 + 32] = recipient_bytes[18];
        hash_bytes[23 + 32] = recipient_bytes[19];
        hash_bytes[24 + 32] = recipient_bytes[20];
        hash_bytes[25 + 32] = recipient_bytes[21];
        hash_bytes[26 + 32] = recipient_bytes[22];
        hash_bytes[27 + 32] = recipient_bytes[23];
        hash_bytes[28 + 32] = recipient_bytes[24];
        hash_bytes[29 + 32] = recipient_bytes[25];
        hash_bytes[30 + 32] = recipient_bytes[26];
        hash_bytes[31 + 32] = recipient_bytes[27];
        hash_bytes[32 + 32] = recipient_bytes[28];
        hash_bytes[33 + 32] = recipient_bytes[29];
        hash_bytes[34 + 32] = recipient_bytes[30];
        hash_bytes[35 + 32] = recipient_bytes[31];

        let content_sha256 = dep::std::hash::sha256(hash_bytes);

        // Convert the content_sha256 to a field element
        let mut v = 1;
        let mut high = 0 as Field;
        let mut low = 0 as Field;
        
        // Unroll loops because otherwise takes forever to compile
        // for i in [15 - i for i in range(16)]:
        //     print('high = high + (content_sha256[{0}] as Field) * v;'.format(i))
        //     print('low = low + (content_sha256[16 + {0}] as Field) * v;'.format(i))
        //     print('v = v * 256;');
        high = high + (content_sha256[15] as Field) * v;
        low = low + (content_sha256[16 + 15] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[14] as Field) * v;
        low = low + (content_sha256[16 + 14] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[13] as Field) * v;
        low = low + (content_sha256[16 + 13] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[12] as Field) * v;
        low = low + (content_sha256[16 + 12] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[11] as Field) * v;
        low = low + (content_sha256[16 + 11] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[10] as Field) * v;
        low = low + (content_sha256[16 + 10] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[9] as Field) * v;
        low = low + (content_sha256[16 + 9] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[8] as Field) * v;
        low = low + (content_sha256[16 + 8] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[7] as Field) * v;
        low = low + (content_sha256[16 + 7] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[6] as Field) * v;
        low = low + (content_sha256[16 + 6] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[5] as Field) * v;
        low = low + (content_sha256[16 + 5] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[4] as Field) * v;
        low = low + (content_sha256[16 + 4] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[3] as Field) * v;
        low = low + (content_sha256[16 + 3] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[2] as Field) * v;
        low = low + (content_sha256[16 + 2] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[1] as Field) * v;
        low = low + (content_sha256[16 + 1] as Field) * v;
        v = v * 256;
        high = high + (content_sha256[0] as Field) * v;
        low = low + (content_sha256[16 + 0] as Field) * v;
        v = v * 256;

        // Abuse that a % p + b % p = (a + b) % p and that low < p
        let content = low + high * v;
        content
    }
}
