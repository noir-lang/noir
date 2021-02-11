#[foreign(schnorr_verify)]
fn verify_signature(_signature :[64]u8, _message : []u8, _public_key_x : Witness, _public_key_y : Witness) -> Witness {}