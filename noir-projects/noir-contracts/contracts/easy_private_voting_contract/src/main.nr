contract EasyPrivateVoting {
    // docs:start:imports
    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, NoteHeader, NoteInterface, NoteGetterOptions, PrivateContext,
        Map, PublicMutable
    };
    use dep::aztec::context::Context;
    // docs:end:imports
    // docs:start:storage_struct
    #[aztec(storage)]
    struct Storage {
        admin: PublicMutable<AztecAddress>, // admin can end vote
        tally: Map<Field, PublicMutable<Field>>, // we will store candidate as key and number of votes as value
        vote_ended: PublicMutable<bool>, // vote_ended is boolean
    }
    // docs:end:storage_struct

    // docs:start:constructor
    #[aztec(public)]
    #[aztec(initializer)] // annotation to mark function as a constructor
    fn constructor(admin: AztecAddress) {
        storage.admin.write(admin);
        storage.vote_ended.write(false);
    }
    // docs:end:constructor

    // docs:start:cast_vote
    #[aztec(private)] // annotation to mark function as private and expose private context
    fn cast_vote(candidate: Field) {
        let secret = context.request_nullifier_secret_key(context.msg_sender()); // get secret key of caller of function
        let nullifier = dep::std::hash::pedersen_hash([context.msg_sender().to_field(), secret.low, secret.high]); // compute nullifier with this secret key so others can't descrypt it
        context.push_new_nullifier(nullifier, 0); // push nullifier
        EasyPrivateVoting::at(context.this_address()).add_to_tally_public(candidate).enqueue(&mut context);
    }
    // docs:end:cast_vote

    // docs:start:add_to_tally_public
    #[aztec(public)]
    #[aztec(internal)]
    fn add_to_tally_public(candidate: Field) {
        assert(storage.vote_ended.read() == false, "Vote has ended"); // assert that vote has not ended
        let new_tally = storage.tally.at(candidate).read() + 1;
        storage.tally.at(candidate).write(new_tally);
    }
    // docs:end:add_to_tally_public

    // docs:start:end_vote
    #[aztec(public)] 
    fn end_vote() {
        assert(storage.admin.read().eq(context.msg_sender()), "Only admin can end votes"); // assert that caller is admin
        storage.vote_ended.write(true);
    }
    // docs:end:end_vote
    // docs:start:get_vote
    unconstrained fn get_vote(candidate: Field) -> pub Field {
        storage.tally.at(candidate).read()
    }
    // docs:end:get_vote
}
