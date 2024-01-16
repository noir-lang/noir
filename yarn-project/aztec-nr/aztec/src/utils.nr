pub fn arr_copy_slice<T, N, M>(src: [T; N], mut dst: [T; M], offset: Field) -> [T; M] {
    for i in 0..dst.len() {
        dst[i] = src[i + offset];
    }
    dst
}

// TODO(#3470): Copied over from https://github.com/AztecProtocol/aztec-packages/blob/a07c4bd47313be6aa604a63f37857eb0136b41ba/yarn-project/noir-protocol-circuits/src/crates/rollup-lib/src/base/base_rollup_inputs.nr#L599
// move to a shared place?

// TODO to radix returns u8, so we cannot use bigger radixes. It'd be ideal to use a radix of the maximum range-constrained integer noir supports
pub fn full_field_less_than(lhs: Field, rhs: Field) -> bool {
    lhs.lt(rhs)
}

pub fn full_field_greater_than(lhs: Field, rhs: Field) -> bool {
    rhs.lt(lhs)
}

struct Reader<N> {
    data: [Field; N],
    offset: Field,
}

impl<N> Reader<N> {
    pub fn new(data: [Field; N]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read(&mut self) -> Field {
        let result = self.data[self.offset];
        self.offset += 1;
        result
    }

    pub fn read_array<K>(&mut self, mut result: [Field; K]) -> [Field; K] {
        for i in 0..K {
            result[i] = self.data[self.offset + i];
        }
        self.offset += K;
        result
    }

    pub fn read_struct<T, K>(&mut self, deserialise: fn([Field; K]) -> T) -> T {
        let result = deserialise(self.read_array([0; K]));
        result
    }

    pub fn read_struct_array<T, K, C>(&mut self, deserialise: fn([Field; K]) -> T, mut result: [T; C]) -> [T; C] {
        for i in 0..C {
            result[i] = self.read_struct(deserialise);
        }
        result
    }

    pub fn finish(self) {
        assert(self.offset == self.data.len(), "Reader did not read all data");
    } 
}
