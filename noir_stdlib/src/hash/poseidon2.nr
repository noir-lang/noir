use crate::default::Default;
use crate::hash::Hasher;

global RATE: u32 = 3;

pub(crate) struct Poseidon2Hasher {
    _state: [Field],
}

impl Hasher for Poseidon2Hasher {
    fn finish(self) -> Field {
        self.finish_ref()
    }

    fn finish_ref(&self) -> Field {
        let input_length = self._state.len();
        let iv: Field = (input_length as Field) * 18446744073709551616; // iv = (self._state.len() << 64)
        let mut state = [0, 0, 0, iv];

        // These for-loops can run for a dynamic number of iterations, however
        // this is not an issue in practice as this code is only reachable within a `comptime` context
        for i in 0..input_length / RATE {
            state[0] += self._state[i * RATE];
            state[1] += self._state[i * RATE + 1];
            state[2] += self._state[i * RATE + 2];
            state = super::poseidon2_permutation(state);
        }

        for i in 0..input_length % RATE {
            state[i] += self._state[i];
        }

        crate::hash::poseidon2_permutation(state)[0]
    }

    fn write(&mut self, input: Field) {
        self._state = self._state.push_back(input);
    }
}

impl Default for Poseidon2Hasher {
    fn default() -> Self {
        Poseidon2Hasher { _state: [].as_vector() }
    }
}
