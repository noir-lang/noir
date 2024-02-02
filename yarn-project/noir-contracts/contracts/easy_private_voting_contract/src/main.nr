contract EasyPrivateVoting {
    // docs:start:imports
    use dep::aztec::{
        protocol_types::{
            abis::function_selector::FunctionSelector,
            address::AztecAddress
        },
        context::{PrivateContext, Context},
        state_vars::{ map::Map, public_state::PublicState}, 
    };
    // docs:end:imports
    // docs:start:storage_struct
    struct Storage {
        admin: PublicState<AztecAddress>, // admin can end vote
        tally: Map<Field, PublicState<Field>>, // we will store candidate as key and number of votes as value
        voteEnded: PublicState<bool>, // voteEnded is boolean
    }
    // docs:end:storage_struct

    // docs:start:constructor
    #[aztec(private)] // annotation to mark function as private and expose private context
    fn constructor(admin: AztecAddress) { // called when contract is deployed
        context.call_public_function(
            // we cannot update public state directly from private function but we can call public function (which queues it)
            context.this_address(),// contract address whose method we want to call
            FunctionSelector::from_signature("_initialize((Field))"), // function selector
            [admin.to_field()] // parameters
        );
    }
    // docs:end:constructor
    // docs:start:initialize
    #[aztec(public)] // annotation to mark function as public and expose public context
    internal fn _initialize(admin: AztecAddress) { // internal - can only be called by contract
        storage.admin.write(admin);
        storage.voteEnded.write(false);
    }
    // docs:end:initialize
    // docs:start:cast_vote
    #[aztec(private)] // annotation to mark function as private and expose private context
    fn cast_vote(candidate: Field) {
        let secret = context.request_nullifier_secret_key(context.msg_sender()); // get secret key of caller of function
        let nullifier = dep::std::hash::pedersen_hash([context.msg_sender().to_field(), secret.low, secret.high]); // compute nullifier with this secret key so others can't descrypt it
        context.push_new_nullifier(nullifier, 0); // push nullifier
        context.call_public_function(
            context.this_address(),
            FunctionSelector::from_signature("add_to_tally_public(Field)"),
            [candidate]
        );
    }
    // docs:end:cast_vote

    // docs:start:add_to_tally_public
    #[aztec(public)]
    internal fn add_to_tally_public(candidate: Field) {
        assert(storage.voteEnded.read() == false, "Vote has ended"); // assert that vote has not ended
        let new_tally = storage.tally.at(candidate).read() + 1;
        storage.tally.at(candidate).write(new_tally);
    }
    // docs:end:add_to_tally_public

    // docs:start:end_vote
    #[aztec(public)] 
    fn end_vote() {
        assert(storage.admin.read().eq(context.msg_sender()), "Only admin can end votes"); // assert that caller is admin
        storage.voteEnded.write(true);
    }
    // docs:end:end_vote
    // docs:start:get_vote
    unconstrained fn get_vote(candidate: Field) -> pub Field {
        storage.tally.at(candidate).read()
    }
    // docs:end:get_vote
    // docs:start:compute_note_hash_and_nullifier
    unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; 0]
    ) -> pub [Field; 4] {
        [0, 0, 0, 0]
    }
    // docs:end:compute_note_hash_and_nullifier
}
