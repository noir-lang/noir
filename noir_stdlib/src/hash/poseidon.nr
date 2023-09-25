mod bn254; // Instantiations of Poseidon for prime field of the same order as BN254

use crate::field::modulus_num_bits;

struct PoseidonConfig<M,N> {
    t: Field, // Width, i.e. state size
    rf: u8, // Number of full rounds; should be even
    rp: u8, // Number of partial rounds
    alpha: Field, // S-box power; depends on the underlying field
    ark: [Field; M], // Additive round keys
    mds: [Field; N] // MDS Matrix in row-major order
}

pub fn config<M,N>(
    t: Field,
    rf: u8,
    rp: u8,
    alpha: Field,
    ark: [Field; M],
    mds: [Field; N])
    -> PoseidonConfig<M,N> {
    // Input checks
    assert(t as u8 * (rf + rp) == ark.len() as u8);
    assert(t * t == mds.len());
    assert(alpha != 0);
    
    PoseidonConfig {t, rf, rp, alpha, ark, mds}
}

// General Poseidon permutation on elements of type Field
fn permute<M,N,O>(
    pos_conf: PoseidonConfig<M, N>,
    mut state: [Field; O])
    -> [Field; O] {
    let PoseidonConfig {t, rf, rp, alpha, ark, mds} = pos_conf;

    assert(t == state.len());
    
    let mut count = 0;

    // for r in 0..rf + rp
    for r in 0..(ark.len()/state.len()) {
        for i in 0..state.len() {
            state[i] = state[i] + ark[count + i];
        } // Shift by round constants
        
        state[0] = state[0].pow_32(alpha);

        // Check whether we are in a full round
        if (r as u8 < rf/2) | (r as u8 >= rf/2 + rp) {
            for i in 1..state.len() {
                state[i] = state[i].pow_32(alpha);
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
    mut state: [Field; O], // Initial state; usually [0; O]
    rate: Field, // Rate
    capacity: Field, // Capacity; usually 1
    msg: [Field; P]) // Arbitrary length message
    -> [Field; O] {
    assert(pos_conf.t == rate + capacity);
    
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


// Check security of sponge instantiation
fn check_security(rate: Field, width: Field, security: Field) -> bool {
    let n = modulus_num_bits();

    ((n-1)*(width-rate)/2) as u8 > security as u8
}

// A*x where A is an n x n matrix in row-major order and x an n-vector
fn apply_matrix<N, M>(a: [Field; M], x: [Field; N]) -> [Field; N] {
    let mut y = x;

    for i in 0..x.len() {
        y[i] = 0;
        for j in 0..x.len() {
            y[i] = y[i] + a[x.len()*i + j]* x[j];            
        }
    }
    
    y
}
