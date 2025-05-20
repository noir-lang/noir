pub struct Reader<let N: u32> {
    data: [Field; N],
    offset: u32,
}

impl<let N: u32> Reader<N> {
    pub fn new(data: [Field; N]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read(&mut self) -> Field {
        let result = self.data[self.offset];
        self.offset += 1;
        result
    }

    pub fn read_u32(&mut self) -> u32 {
        self.read() as u32
    }

    pub fn read_bool(&mut self) -> bool {
        self.read() as bool
    }

    pub fn read_array<let K: u32>(&mut self) -> [Field; K] {
        let mut result = [0; K];
        for i in 0..K {
            result[i] = self.data[self.offset + i];
        }
        self.offset += K;
        result
    }

    pub fn read_struct<T, let K: u32>(&mut self, deserialise: fn([Field; K]) -> T) -> T {
        let result = deserialise(self.read_array());
        result
    }

    pub fn read_struct_array<T, let K: u32, let C: u32>(
        &mut self,
        deserialise: fn([Field; K]) -> T,
        mut result: [T; C],
    ) -> [T; C] {
        for i in 0..C {
            result[i] = self.read_struct(deserialise);
        }
        result
    }

    pub fn finish(self) {
        assert(self.offset == self.data.len(), "Reader did not read all data");
    }
}
