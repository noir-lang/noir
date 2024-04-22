use dep::std::option::Option;

trait MyDeserialize<N> {
    fn deserialize(fields: [field; N]) -> Self;
}

impl MyDeserialize<1> for field {
    fn deserialize(fields: [field; 1]) -> Self {
        fields[0]
    }
}

pub fn storage_read<N>() -> [field; N] {
    dep::std::unsafe::zeroed()
}

struct PublicMutable<T> {
    storage_slot: field,
}

impl<T> PublicMutable<T> {
    pub fn new(storage_slot: field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PublicMutable { storage_slot }
    }

    pub fn read<T_SERIALIZED_LEN>(_self: Self) -> T where T: MyDeserialize<T_SERIALIZED_LEN> {
        // storage_read returns slice here
        let fields: [field; T_SERIALIZED_LEN] = storage_read();
        T::deserialize(fields)
    }
}

fn main(value: field) {
    let ps: PublicMutable<field> = PublicMutable::new(27);

    // error here
    assert(ps.read() == value);
}
