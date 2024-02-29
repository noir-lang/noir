struct Reader<N> {
    data: [Field; N],
    offset: u64,
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

    // TODO(#4394)
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
