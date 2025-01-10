use crate::default::Default;
use crate::hash::Hasher;

comptime global RATE: u32 = 3;

pub struct Poseidon2 {
    cache: [Field; 3],
    state: [Field; 4],
    cache_size: u32,
    squeeze_mode: bool, // 0 => absorb, 1 => squeeze
}

impl Poseidon2 {
    #[no_predicates]
    pub fn hash<let N: u32>(input: [Field; N], message_size: u32) -> Field {
        Poseidon2::hash_internal(input, message_size, message_size != N)
    }

    pub(crate) fn new(iv: Field) -> Poseidon2 {
        let mut result =
            Poseidon2 { cache: [0; 3], state: [0; 4], cache_size: 0, squeeze_mode: false };
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
        self.state = crate::hash::poseidon2_permutation(self.state, 4);
    }

    fn absorb(&mut self, input: Field) {
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

    fn squeeze(&mut self) -> Field {
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
        let two_pow_64 = 18446744073709551616;
        let iv: Field = (in_len as Field) * two_pow_64;
        let mut sponge = Poseidon2::new(iv);
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

pub struct Poseidon2Hasher {
    _state: [Field],
}

impl Hasher for Poseidon2Hasher {
    fn finish(self) -> Field {
        let iv: Field = (self._state.len() as Field) * 18446744073709551616; // iv = (self._state.len() << 64)
        let mut sponge = Poseidon2::new(iv);
        for i in 0..self._state.len() {
            sponge.absorb(self._state[i]);
        }
        sponge.squeeze()
    }

    fn write(&mut self, input: Field) {
        self._state = self._state.push_back(input);
    }
}

impl Default for Poseidon2Hasher {
    fn default() -> Self {
        Poseidon2Hasher { _state: &[] }
    }
}
