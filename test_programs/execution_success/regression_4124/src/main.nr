use dep::std::option::Option;

trait MyDeserialize<N> {
    fn deserialize(fields: [Field; N]) -> Self;
}

impl MyDeserialize<1> for Field {
    fn deserialize(fields: [Field; 1]) -> Self {
        fields[0]
    }
}

pub fn storage_read<N>() -> [Field; N] {
    dep::std::unsafe::zeroed()
}

struct PublicState<T> {
    storage_slot: Field,
}

impl<T> PublicState<T> {
    pub fn new(storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PublicState { storage_slot }
    }

    pub fn read<T_SERIALIZED_LEN>(_self: Self) -> T where T: MyDeserialize<T_SERIALIZED_LEN> {
        // storage_read returns slice here
        let fields: [Field; T_SERIALIZED_LEN] = storage_read();
        T::deserialize(fields)
    }
}

fn main(value: Field) {
    let ps: PublicState<Field> = PublicState::new(27);

    // error here
    assert(ps.read() == value);
}
