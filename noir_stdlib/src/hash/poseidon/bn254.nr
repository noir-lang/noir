// Instantiations of Poseidon constants, permutations and sponge for prime field of the same order as BN254
mod perm;
mod consts;

use crate::hash::poseidon::PoseidonConfig;
use crate::hash::poseidon::apply_matrix;

// Optimised permutation for this particular field; uses hardcoded rf and rp values,
// which should agree with those in pos_conf.
fn permute<M,N,O>(
    pos_conf: PoseidonConfig<M, N>,
    mut state: [Field; O])
    -> [Field; O] {
    let PoseidonConfig {t, rf: config_rf, rp: config_rp, alpha, ark, mds} = pos_conf;
    let rf = 8;
    let rp = [56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68][state.len() - 2];

    constrain t == state.len();
    constrain rf == config_rf as Field;
    constrain rp == config_rp as Field;
    
    let mut count = 0;

    // First half of full rounds
    for _r in 0..rf/2 {
        for i in 0..state.len() {
            state[i] = state[i] + ark[count + i];
        } // Shift by round constants
        
        for i in 0..state.len() {
            state[i] = state[i].pow_32(alpha);
        }
        
        state = apply_matrix(mds, state); // Apply MDS matrix
        count = count + t;
    }

    // Partial rounds
    for _r in 0..rp {
        for i in 0..state.len() {
            state[i] = state[i] + ark[count + i];
        } // Shift by round constants
        
        state[0] = state[0].pow_32(alpha);
        
        state = apply_matrix(mds, state); // Apply MDS matrix
        count = count + t;
    }

    // Second half of full rounds
    for _r in 0..rf/2 {
        for i in 0..state.len() {
            state[i] = state[i] + ark[count + i];
        } // Shift by round constants
        
        for i in 0..state.len() {
            state[i] = state[i].pow_32(alpha);
        }
        
        state = apply_matrix(mds, state); // Apply MDS matrix
        count = count + t;
    }
    
    state
}

// Corresponding absorption.
fn absorb<M,N,O,P>(
    pos_conf: PoseidonConfig<M, N>,
    mut state: [Field; O], // Initial state; usually [0; N]
    rate: comptime Field, // Rate
    capacity: comptime Field, // Capacity; usually 1
    msg: [Field; P] // Arbitrary length message
) -> [Field; O] {

    constrain pos_conf.t == rate + capacity;
    
    let mut i = 0;

    for k in 0..msg.len() {
        // Add current block to state
        state[capacity + i] += msg[k];
        i = i+1;
        
        // Enough to absorb
        if i == rate {
            state = permute(pos_conf, state);
            i = 0;
        }
    }

    // If we have one more block to permute
    if i != 0 {
        state = permute(pos_conf, state);
    }
    
    state
}

// Variable-length Poseidon-128 sponge as suggested in second bullet point of §3 of https://eprint.iacr.org/2019/458.pdf
fn sponge<N>(msg: [Field; N]) -> Field {
    absorb(consts::x5_5_config(), [0;5], 4, 1, msg)[1]
}
