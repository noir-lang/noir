#[foreign(sha256)]
fn sha256(_input : []Witness) -> [2]Witness {}

#[foreign(blake2s)]
fn blake2s(_input : []Witness) -> [2]Witness {}

#[foreign(pedersen)]
fn pedersen(_input : []Witness) -> Witness {}

#[foreign(hash_to_field)]
fn hash_to_field(_input : []Witness) -> Witness {}