#[foreign(sha256)]
fn sha256(_input : []Field) -> [32]u8 {}

#[foreign(blake2s)]
fn blake2s(_input : []Field) -> [32]u8 {}

#[foreign(pedersen)]
fn pedersen(_input : []Field) -> Field {}

#[foreign(hash_to_field)]
fn hash_to_field(_input : []Field) -> Field {}