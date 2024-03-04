#[oracle(avmOpcodeKeccak256)]
pub fn keccak256<N>(input: [Field; N]) -> [Field; 2] {}

#[oracle(avmOpcodePoseidon)]
pub fn poseidon<N>(input: [Field; N]) -> Field {}

#[oracle(avmOpcodeSha256)]
pub fn sha256<N>(input: [Field; N]) -> [Field; 2] {}
