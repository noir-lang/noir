use dep::std::hash::pedersen;
use dep::aztec::context::Context;
use dep::aztec::messaging::get_commitment_getter_data::make_commitment_getter_data;
use dep::aztec::oracle::{
    create_commitment::create_commitment,
    create_l2_to_l1_message::create_l2_to_l1_message,
    get_commitment::get_commitment
};
use dep::aztec::constants_gen::GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET;


// Transparent note represents a note that is created in the clear (public execution), 
// but can only be spent by those that know the preimage of the "secretHash"
struct TransparentNote {
    amount: Field,
    secretHash: Field,
}

impl TransparentNote {
    fn new(amount: Field, secretHash: Field) -> Self {
        Self { amount, secretHash }
    }

    fn new_from_secret(amount: Field, secret: Field) -> Self {
        let secretHash = TransparentNote::compute_secret_hash(secret);
        TransparentNote {
            amount,
            secretHash,
        }
    }

    // Gets the value of the commitment
    // TODO(maddiaa): this will need to be hashed with a slot to keep it unique, which slot to pick? https://github.com/AztecProtocol/aztec-packages/issues/847 
    fn get_commitment(self: Self) -> Field {
        pedersen([self.amount, self.secretHash])[0]
    }

    fn consume_in_secret(self: Self, mut context: Context, root: Field, secret: Field) -> Context {
        // Get the commitment value (before silo)
        let commitment = self.get_commitment();

        // Let the kernel perform the read.
        context = context.push_read_request(commitment);

        // Get the commitment data, (where it is in the db)
        let commitment_oracle_call = get_commitment(commitment);
        let commitment_data = make_commitment_getter_data(commitment_oracle_call, 0);
        // Do we still need to do this with read requests?
        assert(root == commitment_data.root);

        // Calculate the nullifier
        self.emit_nullifier(context, secret, commitment, commitment_data.leaf_index)
    }

    fn compute_secret_hash(secret: Field) -> Field {
        pedersen([GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET, secret])[0]
    }

    fn knows_secret(self, secret: Field) {
        let hash = TransparentNote::compute_secret_hash(secret);
        assert(self.secretHash == hash);
    }

    fn emit_nullifier(_self: Self, mut context: Context, secret: Field, siloed_commitment: Field, index: Field) -> Context {
        // Create a nullifier for the message based on its index in the tree
        
        let nullifier = pedersen([secret, siloed_commitment, index])[0];
        context.push_new_nullifier(nullifier, siloed_commitment)
    }

}