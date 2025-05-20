use crate::{traits::{Deserialize, Empty, Serialize}, utils::{arrays::array_concat, reader::Reader}};

pub struct Log<let N: u32> {
    pub fields: [Field; N],
    // The actual length (number of fields) of the log to be included in the blobs.
    pub length: u32,
}

impl<let N: u32> Log<N> {
    pub fn new(fields: [Field; N], length: u32) -> Self {
        Self { fields, length }
    }
}

impl<let N: u32> Eq for Log<N> {
    fn eq(self, other: Log<N>) -> bool {
        (self.fields == other.fields) & (self.length == other.length)
    }
}

impl<let N: u32> Empty for Log<N> {
    fn empty() -> Log<N> {
        Log { fields: [0; N], length: 0 }
    }
}

impl<let N: u32> Serialize<N + 1> for Log<N> {
    fn serialize(self) -> [Field; N + 1] {
        array_concat(self.fields, [self.length as Field])
    }
}

impl<let N: u32> Deserialize<N + 1> for Log<N> {
    fn deserialize(fields: [Field; N + 1]) -> Log<N> {
        let mut reader = Reader::new(fields);
        let fields = reader.read_array();
        let length = reader.read_u32();
        reader.finish();
        Log { fields, length }
    }
}

#[test]
fn serialization_of_empty_log() {
    let item: Log<5> = Log::empty();
    let serialized = item.serialize();
    let deserialized = Log::deserialize(serialized);
    assert(item.eq(deserialized));
}
