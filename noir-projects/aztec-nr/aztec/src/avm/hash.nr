#[oracle(avmOpcodeKeccak256)]
pub fn keccak256<N>(input: [Field; N]) -> [Field; 2] {}

#[oracle(avmOpcodeSha256)]
pub fn sha256<N>(input: [Field; N]) -> [Field; 2] {}
