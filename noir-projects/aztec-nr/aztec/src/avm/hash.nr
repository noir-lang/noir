#[oracle(keccak256)]
pub fn keccak256<N>(input: [Field; N]) -> [Field; 2] {}

#[oracle(poseidon)]
pub fn poseidon<N>(input: [Field; N]) -> Field {}

#[oracle(sha256)]
pub fn sha256<N>(input: [Field; N]) -> [Field; 2] {}
