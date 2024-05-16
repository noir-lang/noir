// A contract used along with `Parent` contract to test nested calls.
contract DelegatedOn {
    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, NoteHeader, NoteGetterOptions, NoteViewerOptions, PublicMutable,
        PrivateSet, PrivateContext
    };
    use dep::aztec::{protocol_types::grumpkin_point::GrumpkinPoint, keys::getters::{get_npk_m_hash, get_ivpk_m}};
    use dep::value_note::value_note::ValueNote;

    #[aztec(storage)]
    struct Storage {
        current_value: PublicMutable<Field>,
        a_private_value: PrivateSet<ValueNote>,
    }

    #[aztec(private)]
    fn private_set_value(new_value: Field, owner: AztecAddress) -> Field {
        let owner_npk_m_hash = get_npk_m_hash(&mut context, owner);
        let owner_ivpk_m = get_ivpk_m(&mut context, owner);

        let mut note = ValueNote::new(new_value, owner_npk_m_hash);
        storage.a_private_value.insert(&mut note, true, owner_ivpk_m);
        new_value
    }

    #[aztec(public)]
    fn public_set_value(new_value: Field) -> Field {
        storage.current_value.write(new_value);
        new_value
    }

    #[aztec(private)]
    fn get_private_value(amount: Field, owner: AztecAddress) -> pub Field {
        let owner_npk_m_hash = get_npk_m_hash(&mut context, owner);

        // TODO (#6312): This will break with key rotation. Fix this. Will not be able to find any notes after rotating key.
        let mut options = NoteGetterOptions::new();
        options = options.select(ValueNote::properties().value, amount, Option::none()).select(
            ValueNote::properties().npk_m_hash,
            owner_npk_m_hash,
            Option::none()
        ).set_limit(1);
        let notes = storage.a_private_value.get_notes(options);
        notes[0].unwrap_unchecked().value
    }

    unconstrained fn view_public_value() -> pub Field {
        storage.current_value.read()
    }
}

