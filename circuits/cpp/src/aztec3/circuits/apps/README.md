# aztec3::circuits::apps

All code in this dir is for:
- Demonstrating how our users' (developers) apps will need to be written, in terms of:
    - Public inputs
    - Syntax for State Variables & UTXOs
    - Function calls
- To help illustrate ideas to the Noir team
- To allow test app circuits to be mocked-up as quickly and easily as possible, for use in the Kernel & Rollup circuits.




utxo_state_var.insert(note_preimage)
        ↓
Opcodes.UTXO_SSTORE (this=state_var*, note_preimage)
        ↓
exec_ctx got from state_var->exec_ctx
new_note = Note(state_var, new_note_preimage) <- could be done in state_var
exec_ctx->new_notes.push_back(new_note)
NO VISITOR CALLED



utxo_state_var.get(NotePreimage advice)
        ↓
Opcodes.UTXO_LOAD (this=state_var*, advice)
        ↓
oracle got from state_var->exec_ctx->oracle
storage_slot_point got from state_var->storage_slot_point
Grab note_preimages and paths from DB
new_note = Note(state_var, note_preimage)
Compare against advice
- VISITOR CALLED: new_note.constrain_against_advice(advice)
Membership checks (and other checks)
- VISITOR CALLED: new_note.get_commitment()
return new_note



note.remove()
        ↓
Opcodes.UTXO_NULL (state_var*, *this=note)
        ↓
exec_ctx got from state_var->exec_ctx
- VISITOR CALLED:
  - nullifier = note.compute_nullifier() <- could be done in state_var
exec_ctx->new_nullifiers.push_back(nullifier)



utxo_state_var.initialise(note_preimage)
        ↓
Opcodes.UTXO_SSTORE (this=state_var*, note_preimage)
        ↓
exec_ctx got from state_var->exec_ctx
new_note = Note(state_var, new_note_preimage)
exec_ctx->new_notes.push_back(new_note)
NO VISITOR CALLED