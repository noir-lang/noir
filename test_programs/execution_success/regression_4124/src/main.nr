use std::option::Option;

trait MyDeserialize<let N: u32> {
    fn deserialize(fields: [Field; N]) -> Self;
}

impl MyDeserialize<1> for Field {
    fn deserialize(fields: [Field; 1]) -> Self {
        fields[0]
    }
}

pub fn storage_read<let N: u32>() -> [Field; N] {
    std::mem::zeroed()
}

struct PublicMutable<T> {
    storage_slot: Field,
}

impl<T> PublicMutable<T> {
    pub fn new(storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PublicMutable { storage_slot }
    }

    pub fn read<let T_SERIALIZED_LEN: u32>(_self: Self) -> T
    where
        T: MyDeserialize<T_SERIALIZED_LEN>,
    {
        // storage_read returns slice here
        let fields: [Field; T_SERIALIZED_LEN] = storage_read();
        T::deserialize(fields)
    }
}

fn main(value: Field) {
    let ps: PublicMutable<Field> = PublicMutable::new(27);

    // error here
    assert(ps.read() == value);
}
