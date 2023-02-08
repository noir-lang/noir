mod bn254; // Instantiations of Poseidon for prime field of the same order as BN254

use crate::array;
use crate::pow_32;
use crate::field::modulus_num_bits;

struct PoseidonConfig<M,N>
{
    t: comptime Field, // Width, i.e. state size
    rf: comptime u8, // Number of full rounds; should be even
    rp: comptime u8, // Number of partial rounds
    alpha: comptime Field, // S-box power; depends on the underlying field
    ark: [Field; M], // Additive round keys
    mds: [Field; N] // MDS Matrix in row-major order
}

fn config<M,N>(
    t: comptime Field,
    rf: comptime u8,
    rp: comptime u8,
    alpha: comptime Field,
    ark: [Field; M],
    mds: [Field; N])
    -> PoseidonConfig<M,N>
{
    // Input checks
    constrain t as u8 * (rf + rp) == array::len(ark) as u8;
    constrain t * t == array::len(mds);
    constrain alpha > 0;
    
    PoseidonConfig {t, rf, rp, alpha, ark, mds}
}

// General Poseidon permutation on elements of type Field
fn permute<M,N,O>(
    pos_conf: PoseidonConfig<M, N>,
    mut state: [Field; O])
    -> [Field; O]
{
    let PoseidonConfig {t, rf, rp, alpha, ark, mds} = pos_conf;

    constrain t == array::len(state);
    
    let mut count = 0;

    // for r in 0..rf + rp
    for r in 0..(array::len(ark)/array::len(state)) {
        for i in 0..array::len(state) {
            state[i] = state[i] + ark[count + i];
        } // Shift by round constants
        
        state[0] = pow_32(state[0], alpha);

        // Check whether we are in a full round
        if (r as u8 < rf/2) | (r as u8 >= rf/2 + rp) {
            for i in 1..array::len(state) {
                state[i] = pow_32(state[i], alpha);
            }
        }
        
        state = apply_matrix(mds, state); // Apply MDS matrix
        count = count + t;
    }

    state
}

// Absorption. Fully absorbs input message.
fn absorb<M,N,O,P>(
    pos_conf: PoseidonConfig<M, N>,
    mut state: [Field; O], // Initial state; usually [0; N]
    rate: comptime Field, // Rate
    capacity: comptime Field, // Capacity; usually 1
    msg: [Field; P]) // Arbitrary length message
    -> [Field; O]
{

    constrain pos_conf.t == rate + capacity;
    
    let mut i = 0;

    for k in 0..array::len(msg) {
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
    if i > 0 {
        state = permute(pos_conf, state);
    }
    
    state
}


// Check security of sponge instantiation
fn check_security(rate: Field, width: Field, security: Field) -> bool
{
    let n = modulus_num_bits();

    ((n-1)*(width-rate)/2) as u8 > security as u8
}

// A*x where A is an n x n matrix in row-major order and x an n-vector
fn apply_matrix<N>(a: [Field], x: [Field; N]) -> [Field; N]
{
    let mut y = x;

    for i in 0..array::len(x) {
        y[i] = 0;
        for j in 0..array::len(x) {
            y[i] = y[i] + a[array::len(x)*i + j]* x[j];            
        }
    }
    
    y
}
