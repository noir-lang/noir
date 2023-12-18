use dep::protocol_types::{
    constants::GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET,
    hash::pedersen_hash,
};

pub fn compute_secret_hash(secret: Field) -> Field {
    // TODO(#1205) This is probably not the right index to use
    pedersen_hash([secret], GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET)
}
