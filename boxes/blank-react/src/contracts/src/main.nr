contract Blank {
    use dep::aztec::{
        protocol_types::address::AztecAddress,
        state_vars::{singleton::Singleton, map::Map},
        context::{PrivateContext, PublicContext, Context},
        note::{
            utils as note_utils,
            note_interface::NoteInterface,
            note_header::NoteHeader,
        },
    };

    use dep::value_note::value_note::{ValueNote, VALUE_NOTE_LEN};

    struct Storage {
        numbers: Map<AztecAddress, Singleton<ValueNote>>,
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

    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        note_type_id: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
        note_utils::compute_note_hash_and_nullifier(ValueNote::deserialize_content, note_header, serialized_note)
    }
}
