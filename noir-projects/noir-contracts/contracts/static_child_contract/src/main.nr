// A contract used along with `StaticParent` contract to test static calls.
contract StaticChild {
    use dep::aztec::prelude::{AztecAddress, FunctionSelector, PublicMutable, PrivateSet, PrivateContext, Deserialize};

    use dep::aztec::{
        context::{PublicContext, gas::GasOpts}, protocol_types::{abis::{call_context::CallContext}},
        note::{note_getter_options::NoteGetterOptions, note_header::NoteHeader},
        keys::getters::{get_npk_m_hash, get_ivpk_m}
    };
    use dep::value_note::value_note::ValueNote;

    #[aztec(storage)]
    struct Storage {
        current_value: PublicMutable<Field>,
        a_private_value: PrivateSet<ValueNote>,
    }

    // Returns base_value + chain_id + version + block_number + timestamp statically
    #[aztec(public)]
    #[aztec(view)]
    fn pub_get_value(base_value: Field) -> Field {
        let return_value = base_value
            + context.chain_id()
            + context.version()
            + context.block_number()
            + context.timestamp() as Field;

        return_value
    }

    // Sets `current_value` to `new_value`
    #[aztec(public)]
    fn pub_set_value(new_value: Field) -> Field {
        storage.current_value.write(new_value);
        context.emit_unencrypted_log(new_value);

        new_value
    }

    // View function that attempts to modify state. Should always fail regardless how it's called.
    #[aztec(private)]
    #[aztec(view)]
    fn private_illegal_set_value(new_value: Field, owner: AztecAddress) -> Field {
        let owner_npk_m_hash = get_npk_m_hash(&mut context, owner);
        let owner_ivpk_m = get_ivpk_m(&mut context, owner);
        let mut note = ValueNote::new(new_value, owner_npk_m_hash);
        storage.a_private_value.insert(&mut note, true, owner_ivpk_m);
        new_value
    }

    // Modify a note
    #[aztec(private)]
    fn private_set_value(new_value: Field, owner: AztecAddress) -> Field {
        let owner_npk_m_hash = get_npk_m_hash(&mut context, owner);
        let owner_ivpk_m = get_ivpk_m(&mut context, owner);
        let mut note = ValueNote::new(new_value, owner_npk_m_hash);
        storage.a_private_value.insert(&mut note, true, owner_ivpk_m);
        new_value
    }

    // Retrieve note value statically
    #[aztec(private)]
    #[aztec(view)]
    fn private_get_value(amount: Field, owner: AztecAddress) -> Field {
        let owner_npk_m_hash = get_npk_m_hash(&mut context, owner);
        let mut options = NoteGetterOptions::new();
        options = options.select(ValueNote::properties().value, amount, Option::none()).select(
            ValueNote::properties().npk_m_hash,
            owner_npk_m_hash,
            Option::none()
        ).set_limit(1);
        let notes = storage.a_private_value.get_notes(options);
        notes[0].unwrap_unchecked().value
    }

    // Increments `current_value` by `new_value`
    #[aztec(public)]
    fn pub_inc_value(new_value: Field) -> Field {
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        context.emit_unencrypted_log(new_value);

        new_value
    }

    // View function that attempts to modify state. Should always fail regardless how it's called.
    #[aztec(public)]
    #[aztec(view)]
    fn pub_illegal_inc_value(new_value: Field) -> Field {
        let old_value = storage.current_value.read();
        storage.current_value.write(old_value + new_value);
        context.emit_unencrypted_log(new_value);

        new_value
    }
}
