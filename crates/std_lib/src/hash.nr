#[foreign(sha256)]
fn sha256(_input : [u8]) -> [u8; 32] {}

#[foreign(blake2s)]
fn blake2s(_input : [u8]) -> [u8; 32] {}

#[foreign(pedersen)]
fn pedersen(_input : [Field]) -> [Field; 2] {}

#[foreign(hash_to_field)]
fn hash_to_field(_input : [Field]) -> Field {}
