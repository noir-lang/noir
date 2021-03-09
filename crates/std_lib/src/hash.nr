#[foreign(sha256)]
fn sha256(_input : []Field) -> [2]Field {}

#[foreign(blake2s)]
fn blake2s(_input : []Field) -> [2]Field {}

#[foreign(pedersen)]
fn pedersen(_input : []Field) -> Field {}

#[foreign(hash_to_field)]
fn hash_to_field(_input : []Field) -> Field {}