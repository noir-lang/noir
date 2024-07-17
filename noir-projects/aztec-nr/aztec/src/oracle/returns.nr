#[oracle(packReturns)]
unconstrained fn pack_returns_oracle<let N: u32>(_returns: [Field]) -> Field {}

unconstrained pub fn pack_returns(returns: [Field]) {
    let _unused = pack_returns_oracle(returns);
}

#[oracle(unpackReturns)]
unconstrained fn unpack_returns_oracle<let N: u32>(_return_hash: Field) -> [Field; N] {}

unconstrained pub fn unpack_returns<let N: u32>(return_hash: Field) -> [Field; N] {
    unpack_returns_oracle(return_hash)
}
