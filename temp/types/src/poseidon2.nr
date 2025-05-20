use crate::constants::TWO_POW_64;

// NB: This is a clone of noir/noir-repo/noir_stdlib/src/hash/poseidon2.nr
// It exists as we sometimes need to perform custom absorption, but the stdlib version
// has a private absorb() method (it's also designed to just be a hasher)
// Can be removed when standalone noir poseidon lib exists: See noir#6679

comptime global RATE: u32 = 3;

pub struct Poseidon2Sponge {
    pub cache: [Field; 3],
    pub state: [Field; 4],
    pub cache_size: u32,
    pub squeeze_mode: bool, // 0 => absorb, 1 => squeeze
}

impl Poseidon2Sponge {
    #[no_predicates]
    pub fn hash<let N: u32>(input: [Field; N], message_size: u32) -> Field {
        Poseidon2Sponge::hash_internal(input, message_size, message_size != N)
    }

    pub(crate) fn new(iv: Field) -> Poseidon2Sponge {
        let mut result =
            Poseidon2Sponge { cache: [0; 3], state: [0; 4], cache_size: 0, squeeze_mode: false };
        result.state[RATE] = iv;
        result
    }

    fn perform_duplex(&mut self) {
        // add the cache into sponge state
        for i in 0..RATE {
            // We effectively zero-pad the cache by only adding to the state
            // cache that is less than the specified `cache_size`
            if i < self.cache_size {
                self.state[i] += self.cache[i];
            }
        }
        self.state = std::hash::poseidon2_permutation(self.state, 4);
    }

    pub fn absorb(&mut self, input: Field) {
        assert(!self.squeeze_mode);
        if self.cache_size == RATE {
            // If we're absorbing, and the cache is full, apply the sponge permutation to compress the cache
            self.perform_duplex();
            self.cache[0] = input;
            self.cache_size = 1;
        } else {
            // If we're absorbing, and the cache is not full, add the input into the cache
            self.cache[self.cache_size] = input;
            self.cache_size += 1;
        }
    }

    pub fn squeeze(&mut self) -> Field {
        assert(!self.squeeze_mode);
        // If we're in absorb mode, apply sponge permutation to compress the cache.
        self.perform_duplex();
        self.squeeze_mode = true;

        // Pop one item off the top of the permutation and return it.
        self.state[0]
    }

    fn hash_internal<let N: u32>(
        input: [Field; N],
        in_len: u32,
        is_variable_length: bool,
    ) -> Field {
        let iv: Field = (in_len as Field) * TWO_POW_64;
        let mut sponge = Poseidon2Sponge::new(iv);
        for i in 0..input.len() {
            if i < in_len {
                sponge.absorb(input[i]);
            }
        }

        // In the case where the hash preimage is variable-length, we append `1` to the end of the input, to distinguish
        // from fixed-length hashes. (the combination of this additional field element + the hash IV ensures
        // fixed-length and variable-length hashes do not collide)
        if is_variable_length {
            sponge.absorb(1);
        }
        sponge.squeeze()
    }
}
