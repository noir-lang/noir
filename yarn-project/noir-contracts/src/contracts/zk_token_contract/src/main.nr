contract PrivateToken {
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::Outputs;
    use dep::aztec3::notes::value_note::Note;
    use dep::aztec3::types::point::Point;
    use dep::aztec3::state_vars::map::Map;
    use dep::aztec3::state_vars::set::Set;

    //*********************************/
    global balances = Map { storage_slot: 1 };
    // Should eventually be replaced by:
    // `balances: Mapping<Field, Set<Note>>;`
    //*********************************/

    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        inputs: pub Inputs,
        //*********************************/

        // initial_supply: pub Field, 
        // owner: pub Point,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        //*********************************/
        // Should eventually be hidden:
        let initial_supply = inputs.args[0];
        let owner = Point::new(inputs.args[1], inputs.args[2]);

        let mut outputs = Outputs::new();
        //*********************************/

        let owner_balance = balances.at(owner.x);
        let note = Note::new(initial_supply, owner, 0);

        owner_balance.insert(outputs, note)
            .finish(inputs)
    }


    fn mint(
        //*********************************/
        // Should eventually be hidden:
        inputs: pub Inputs,
        //*********************************/

        // amount: pub Field, 
        // owner: pub Point,
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        //*********************************/
        // Should eventually be hidden:
        let amount = inputs.args[0];
        let owner = Point::new(inputs.args[1], inputs.args[2]);

        let mut outputs = Outputs::new();
        //*********************************/

        let owner_balance = balances.at(owner.x);
        let note = Note::new(amount, owner, 0);

        owner_balance.insert(outputs, note)
            .finish(inputs)
    }


    fn transfer(
        //*********************************/
        // Should eventually be hidden:
        inputs: pub Inputs,
        //*********************************/
        // 
        // amount: pub Field, 
        // sender: pub Point, 
        // recipient: pub Point
    ) -> pub [Field; dep::aztec3::abi::PUBLIC_INPUTS_LENGTH] {
        //*********************************/
        // Should eventually be hidden:
        let amount = inputs.args[0];
        let sender = Point::new(inputs.args[1], inputs.args[2]);
        let recipient = Point::new(inputs.args[3], inputs.args[4]);

        let outputs1 = Outputs::new();
        //*********************************/

        let sender_balance = balances.at(sender.x);
        let (mut outputs, (note1, note2)) = sender_balance.get_2(inputs, outputs1);

        let constrain_note = |note: Note, sender_forwarded: Point| {
            if note.is_real {
                constrain sender_forwarded.x == note.owner.x;
                constrain sender_forwarded.y == note.owner.y;
            } else {
                constrain note.value == 0;
            }
        };

        // TODO: Removing this return breaks the circuit (???)
        outputs.return_values = outputs.return_values.push(note1.is_real as Field);
        outputs.return_values = outputs.return_values.push(note2.is_real as Field);

        constrain_note(note1, sender);
        constrain_note(note2, sender);

        let sum = note1.value + note2.value;
        constrain sum as u64 >= amount as u64;
        let change_value = sum - amount;

        outputs = sender_balance.remove(inputs, outputs, note1);
        outputs = sender_balance.remove(inputs, outputs, note2);

        let recipient_note = Note::new(amount, recipient, 0);
        let change_note = Note::new(change_value, sender, 1);

        let recipient_balance = balances.at(recipient.x);

        outputs = recipient_balance.insert(outputs, recipient_note);
        outputs = sender_balance.insert(outputs, change_note);


        outputs.finish(inputs)
    }

}
