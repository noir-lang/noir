// A contract used along with `Parent` contract to test nested calls.
contract DelegatedOn {
    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, NoteHeader, NoteGetterOptions, NoteViewerOptions, PublicMutable,
        PrivateSet, PrivateContext, Map
    };
    use dep::aztec::encrypted_logs::encrypted_note_emission::encode_and_encrypt_note;
    use dep::value_note::value_note::ValueNote;

    #[aztec(storage)]
    struct Storage {
        current_value: PublicMutable<Field>,
        a_map_with_private_values: Map<AztecAddress, PrivateSet<ValueNote>>,
    }

    #[aztec(private)]
    fn private_set_value(new_value: Field, owner: AztecAddress) -> Field {
        let header = context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(&mut context, owner);

        let mut note = ValueNote::new(new_value, owner_npk_m_hash);
        storage.a_map_with_private_values.at(owner).insert(&mut note).emit(encode_and_encrypt_note(&mut context, context.msg_sender(), owner));
        new_value
    }

    #[aztec(public)]
    fn public_set_value(new_value: Field) -> Field {
        storage.current_value.write(new_value);
        new_value
    }

    #[aztec(private)]
    fn get_private_value(amount: Field, owner: AztecAddress) -> pub Field {
        let mut options = NoteGetterOptions::new();
        options = options.select(ValueNote::properties().value, amount, Option::none()).set_limit(1);
        let notes = storage.a_map_with_private_values.at(owner).get_notes(options);
        notes.get_unchecked(0).value
    }

    unconstrained fn view_public_value() -> pub Field {
        storage.current_value.read()
    }
}

