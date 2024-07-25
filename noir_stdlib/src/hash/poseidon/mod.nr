mod bn254; // Instantiations of Poseidon for prime field of the same order as BN254
use crate::field::modulus_num_bits;
use crate::hash::Hasher;
use crate::default::Default;

// A config struct defining the parameters of the Poseidon instance to use.
//
// A thorough writeup of this method (along with an unoptimized method) can be found at: https://spec.filecoin.io/algorithms/crypto/poseidon/
struct PoseidonConfig<let T: u32, let N: u32, let X: u32> { 
    // State width, should be equal to `T`
    t: Field,
    // Number of full rounds. should be even
    rf: u8,
    // Number of partial rounds
    rp: u8,
     // S-box power; depends on the underlying field
    alpha: Field,
    // The round constants for the
    round_constants: [Field; N],
    // The MDS matrix for the Poseidon instance
    mds: [[Field; T]; T],
    // An MDS matrix which also applies a basis transformation which allows
    // sparse matrices to be used for the partial rounds.
    //
    // This should be applied instead of `mds` in the final full round.
    presparse_mds: [[Field; T]; T],
    // A set of sparse matrices used for MDS mixing for the partial rounds.
    sparse_mds: [Field; X],
}

pub fn config<let T: u32, let N: u32, let X: u32>(
    t: Field,
    rf: u8,
    rp: u8,
    alpha: Field,
    round_constants: [Field; N],
    mds: [[Field; T]; T],
    presparse_mds: [[Field; T]; T],
    sparse_mds: [Field; X]
) -> PoseidonConfig<T, N, X> {
    // Input checks
    assert_eq(rf & 1, 0);
    assert_eq((t as u8) * rf + rp, N as u8);
    assert_eq(t, T as Field);
    assert(alpha != 0);

    PoseidonConfig { t, rf, rp, alpha, round_constants, mds, presparse_mds, sparse_mds }
}

pub fn permute<let T: u32, let N: u32, let X: u32>(
    pos_conf: PoseidonConfig<T, N, X>,
    mut state: [Field; T]
) -> [Field; T] {
    let PoseidonConfig {t, rf, rp, alpha, round_constants, mds, presparse_mds, sparse_mds } = pos_conf;

    for i in 0..state.len() {
        state[i] += round_constants[i];
    }

    for _r in 0..rf / 2 - 1 {
        state = sigma(state);
        for i in 0..T {
            state[i] += round_constants[T * (_r + 1) as u32 + i];
        }
        state = apply_matrix(mds, state);
    }

    state = sigma(state);
    for i in 0..T {
        state[i] += round_constants[T * (rf / 2) as u32 + i];
    }
    state = apply_matrix(presparse_mds, state);

    for i in 0..T {
        crate::as_witness(state[i]);
    }

    for _r in 0..rp {
        state[0] = state[0].pow_32(alpha);
        state[0] += round_constants[(rf/2 + 1) as u32 * T + _r as u32];
        crate::as_witness(state[0]);
        {
            let mut newState0 = 0;
            for j in 0..T {
                newState0 += sparse_mds[(T * 2 - 1) * _r as u32 + j as u32] * state[j];
            }
            for k in 1..T {
                state[k] += state[0] * sparse_mds[(t * 2 - 1) as u32 * _r as u32 + T + k - 1];
            }
            state[0] = newState0;

            if (_r & 1 == 0) {
                for k in 1..T {
                    crate::as_witness(state[k]);
                }
            }
        }
    }

    for _r in 0..rf / 2 - 1 {
        state = sigma(state);
        for i in 0..state.len() {
            state[i] += round_constants[(rf/2+1) as u32 * T + rp as u32 + (_r as u32) * T + i];
        }
        state = apply_matrix(mds, state);
    }

    state = sigma(state);
    state = apply_matrix(mds, state);

    state
}

// Performs matrix multiplication on a vector
fn apply_matrix<let N: u32>(matrix: [[Field; N]; N], vec: [Field; N]) -> [Field; N] {
    let mut out = [0; N];

    for i in 0..N {
        for j in 0..N {
            out[i] += vec[j] * matrix[j][i];
        }
    }

    out
}

// Corresponding absorption.
fn absorb<let T: u32, let N: u32, let X: u32, let O: u32>(
    pos_conf: PoseidonConfig<T, N, X>,
    // Initial state; usually [0; O]
    mut state: [Field; T],
    rate: Field,
    capacity: Field,
    msg: [Field; O] // Arbitrary length message
) -> [Field; T] {
    assert_eq(pos_conf.t, rate + capacity);

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

fn sigma<let O: u32>(x: [Field; O]) -> [Field; O] {
    let mut y = x;
    for i in 0..O {
        let t = y[i];
        let tt = t * t;
        let tttt = tt * tt;
        y[i] *= tttt;
    }
    y
}

// Check security of sponge instantiation
fn check_security(rate: Field, width: Field, security: Field) -> bool {
    let n = modulus_num_bits();

    ((n - 1) as Field * (width - rate) / 2) as u8 > security as u8
}

struct PoseidonHasher{
    _state: [Field],
}

impl Hasher for PoseidonHasher {
    #[field(bn254)]
    fn finish(self) -> Field {
        let mut result = 0;
        let len = self._state.len();
        assert(len < 16);
        if len == 1 {
            result = bn254::hash_1([self._state[0]]);
        } 
        if len == 2 {
            result = bn254::hash_2([self._state[0],self._state[1]]);
        } 
        if len == 3 {
            result = bn254::hash_3([self._state[0],self._state[1],self._state[2]]);
        } 
        if len == 4 {
            result = bn254::hash_4([self._state[0],self._state[1],self._state[2],self._state[3]]);
        } 
        if len == 5 {
            result = bn254::hash_5([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4]]);
        }
        if len == 6 {
            result = bn254::hash_6([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5]]);
        }
        if len == 7 {
            result = bn254::hash_7([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6]]);
        }
        if len == 8 {
            result = bn254::hash_8([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7]]);
        }
        if len == 9 {
            result = bn254::hash_9([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7], self._state[8]]);
        }
        if len == 10 {
            result = bn254::hash_10([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7], self._state[8], self._state[9]]);
        }
        if len == 11 {
            result = bn254::hash_11([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7], self._state[8], self._state[9], self._state[10]]);
        }
        if len == 12 {
            result = bn254::hash_12([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7], self._state[8], self._state[9], self._state[10], self._state[11]]);
        }
        if len == 13 {
            result = bn254::hash_13([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7], self._state[8], self._state[9], self._state[10], self._state[11], self._state[12]]);
        }
        if len == 14 {
            result = bn254::hash_14([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7], self._state[8], self._state[9], self._state[10], self._state[11], self._state[12], self._state[13]]);
        }
        if len == 15 {
            result = bn254::hash_15([self._state[0],self._state[1],self._state[2],self._state[3],self._state[4], self._state[5], self._state[6], self._state[7], self._state[8], self._state[9], self._state[10], self._state[11], self._state[12], self._state[13], self._state[14]]);
        }
        
        result
    }

    fn write(&mut self, input: Field){
        self._state = self._state.push_back(input);
    }
}

impl Default for PoseidonHasher{
    fn default() -> Self{
        PoseidonHasher{
            _state: &[],
        }
    }
}
