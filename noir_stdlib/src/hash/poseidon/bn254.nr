// Instantiations of Poseidon constants, permutations and sponge for prime field of the same order as BN254
mod perm;
mod consts;

// Variable-length Poseidon-128 sponge as suggested in second bullet point of §3 of https://eprint.iacr.org/2019/458.pdf
fn sponge<N>(msg: [Field; N]) -> Field // Poseidon sponge (absorption) with rate 4 and width 5
{
    crate::hash::poseidon::absorb(consts::rf, consts::rp()[3], consts::alpha, consts::ark5(), consts::mds5(), [0;5], 4, 1, msg)[1]
}
