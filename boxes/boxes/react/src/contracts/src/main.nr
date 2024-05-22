contract BoxReact {
    use dep::aztec::prelude::{AztecAddress, PrivateMutable, Map, NoteInterface, NoteHeader};
    use dep::aztec::protocol_types::grumpkin_point::GrumpkinPoint;

    use dep::value_note::value_note::{ValueNote, VALUE_NOTE_LEN};

    #[aztec(storage)]
    struct Storage {
        numbers: Map<AztecAddress, PrivateMutable<ValueNote>>,
    }

    #[aztec(private)]
    #[aztec(initializer)]
    fn constructor(number: Field, owner: AztecAddress, owner_npk_m_hash: Field, owner_ivpk_m: GrumpkinPoint) {
        let numbers = storage.numbers;
        let mut new_number = ValueNote::new(number, owner_npk_m_hash);
        numbers.at(owner).initialize(&mut new_number, true, owner_ivpk_m);
    }

    #[aztec(private)]
    fn setNumber(number: Field, owner: AztecAddress, owner_npk_m_hash: Field, owner_ivpk_m: GrumpkinPoint) {
        let numbers = storage.numbers;
        let mut new_number = ValueNote::new(number, owner_npk_m_hash);
        numbers.at(owner).replace(&mut new_number, true, owner_ivpk_m);
    }

    unconstrained fn getNumber(owner: AztecAddress) -> pub ValueNote {
        let numbers = storage.numbers;
        numbers.at(owner).view_note()
    }
}
