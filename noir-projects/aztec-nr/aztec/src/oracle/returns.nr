#[oracle(packReturns)]
fn pack_returns_oracle<N>(_returns: [Field]) -> Field {}

unconstrained pub fn pack_returns(returns: [Field]) {
    let _unused = pack_returns_oracle(returns);
}

#[oracle(unpackReturns)]
fn unpack_returns_oracle<N>(_return_hash: Field) -> [Field; N] {}

unconstrained pub fn unpack_returns<N>(return_hash: Field) -> [Field; N] {
    unpack_returns_oracle(return_hash)
}
