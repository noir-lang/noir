contract Vanilla {
    use dep::aztec::{
        protocol_types::address::AztecAddress, state_vars::{PrivateMutable, Map},
        note::{utils as note_utils, note_interface::NoteInterface, note_header::NoteHeader}
    };

    use dep::value_note::value_note::{ValueNote, VALUE_NOTE_LEN};

    struct Storage {
        numbers: Map<AztecAddress, PrivateMutable<ValueNote>>,
    }

    #[aztec(private)]
    fn constructor(number: Field, owner: AztecAddress) {
        let numbers = storage.numbers;
        let mut new_number = ValueNote::new(number, owner);
        numbers.at(owner).initialize(&mut new_number, true);
    }

    #[aztec(private)]
    fn setNumber(number: Field, owner: AztecAddress) {
        let numbers = storage.numbers;
        let mut new_number = ValueNote::new(number, owner);
        numbers.at(owner).replace(&mut new_number, true);
    }

    unconstrained fn getNumber(owner: AztecAddress) -> pub ValueNote {
        let numbers = storage.numbers;
        numbers.at(owner).view_note()
    }
}
