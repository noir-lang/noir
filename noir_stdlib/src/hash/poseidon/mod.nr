pub mod bn254; // Instantiations of Poseidon for prime field of the same order as BN254
use crate::default::Default;
use crate::hash::Hasher;

// A config struct defining the parameters of the Poseidon instance to use.
//
// A thorough writeup of this method (along with an unoptimized method) can be found at: https://spec.filecoin.io/algorithms/crypto/poseidon/
pub struct PoseidonConfig<let T: u32, let N: u32, let X: u32> {
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
    sparse_mds: [Field; X],
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
    mut state: [Field; T],
) -> [Field; T] {
    let PoseidonConfig { t, rf, rp, alpha, round_constants, mds, presparse_mds, sparse_mds } =
        pos_conf;

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
        state[0] += round_constants[(rf / 2 + 1) as u32 * T + _r as u32];
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
            state[i] += round_constants[(rf / 2 + 1) as u32 * T + rp as u32 + (_r as u32) * T + i];
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
    msg: [Field; O], // Arbitrary length message
) -> [Field; T] {
    assert_eq(pos_conf.t, rate + capacity);

    let mut i = 0;

    for k in 0..msg.len() {
        // Add current block to state
        state[capacity + i] += msg[k];
        i = i + 1;
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

pub struct PoseidonHasher {
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
            result = bn254::hash_2([self._state[0], self._state[1]]);
        }
        if len == 3 {
            result = bn254::hash_3([self._state[0], self._state[1], self._state[2]]);
        }
        if len == 4 {
            result = bn254::hash_4([self._state[0], self._state[1], self._state[2], self._state[3]]);
        }
        if len == 5 {
            result = bn254::hash_5([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
            ]);
        }
        if len == 6 {
            result = bn254::hash_6([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
            ]);
        }
        if len == 7 {
            result = bn254::hash_7([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
            ]);
        }
        if len == 8 {
            result = bn254::hash_8([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
            ]);
        }
        if len == 9 {
            result = bn254::hash_9([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
                self._state[8],
            ]);
        }
        if len == 10 {
            result = bn254::hash_10([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
                self._state[8],
                self._state[9],
            ]);
        }
        if len == 11 {
            result = bn254::hash_11([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
                self._state[8],
                self._state[9],
                self._state[10],
            ]);
        }
        if len == 12 {
            result = bn254::hash_12([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
                self._state[8],
                self._state[9],
                self._state[10],
                self._state[11],
            ]);
        }
        if len == 13 {
            result = bn254::hash_13([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
                self._state[8],
                self._state[9],
                self._state[10],
                self._state[11],
                self._state[12],
            ]);
        }
        if len == 14 {
            result = bn254::hash_14([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
                self._state[8],
                self._state[9],
                self._state[10],
                self._state[11],
                self._state[12],
                self._state[13],
            ]);
        }
        if len == 15 {
            result = bn254::hash_15([
                self._state[0],
                self._state[1],
                self._state[2],
                self._state[3],
                self._state[4],
                self._state[5],
                self._state[6],
                self._state[7],
                self._state[8],
                self._state[9],
                self._state[10],
                self._state[11],
                self._state[12],
                self._state[13],
                self._state[14],
            ]);
        }

        result
    }

    #[inline_always]
    fn write(&mut self, input: Field) {
        self._state = self._state.push_back(input);
    }
}

impl Default for PoseidonHasher {
    fn default() -> Self {
        PoseidonHasher { _state: &[] }
    }
}

mod poseidon_tests {
    use crate::hash::poseidon;

    #[test]
    fn reference_impl_test_vectors() {
        // hardcoded test vectors from https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/master/code/test_vectors.txt
        {
            let mut state = [0, 1, 2];
            let mut expected = [
                0x115cc0f5e7d690413df64c6b9662e9cf2a3617f2743245519e19607a4417189a,
                0x0fca49b798923ab0239de1c9e7a4a9a2210312b6a2f616d18b5a87f9b628ae29,
                0x0e7ae82e40091e63cbd4f16a6d16310b3729d4b6e138fcf54110e2867045a30c,
            ];
            assert_eq(
                expected,
                poseidon::bn254::perm::x5_3(state),
                "Failed to reproduce output for [0, 1, 2]",
            );
        }
        {
            let mut state = [0, 1, 2, 3, 4];
            let mut expected = [
                0x299c867db6c1fdd79dcefa40e4510b9837e60ebb1ce0663dbaa525df65250465,
                0x1148aaef609aa338b27dafd89bb98862d8bb2b429aceac47d86206154ffe053d,
                0x24febb87fed7462e23f6665ff9a0111f4044c38ee1672c1ac6b0637d34f24907,
                0x0eb08f6d809668a981c186beaf6110060707059576406b248e5d9cf6e78b3d3e,
                0x07748bc6877c9b82c8b98666ee9d0626ec7f5be4205f79ee8528ef1c4a376fc7,
            ];
            assert_eq(
                expected,
                poseidon::bn254::perm::x5_5(state),
                "Failed to reproduce output for [0, 1, 2, 3, 4]",
            );
        }
    }
}
