use dep::protocol_types::{address::AztecAddress, traits::Deserialize};

#[oracle(storageRead)]
unconstrained fn storage_read_oracle<let N: u32>(address: Field, storage_slot: Field, block_number: Field, length: Field) -> [Field; N] {}

unconstrained pub fn raw_storage_read<let N: u32>(
    address: AztecAddress,
    storage_slot: Field,
    block_number: u32
) -> [Field; N] {
    storage_read_oracle(
        address.to_field(),
        storage_slot,
        block_number as Field,
        N as Field
    )
}

unconstrained pub fn storage_read<T, let N: u32>(
    address: AztecAddress,
    storage_slot: Field,
    block_number: u32
) -> T where T: Deserialize<N> {
    T::deserialize(raw_storage_read(address, storage_slot, block_number))
}

mod tests {
    use crate::oracle::storage::{raw_storage_read, storage_read};
    use dep::protocol_types::address::AztecAddress;

    use std::test::OracleMock;
    use crate::test::mocks::mock_struct::MockStruct;

    global address = AztecAddress::from_field(29);
    global slot = 7;
    global block_number = 17;

    #[test]
    fn test_raw_storage_read() {
        let written = MockStruct { a: 13, b: 42 };

        let _ = OracleMock::mock("storageRead").returns(written.serialize());

        let read: [Field; 2] = raw_storage_read(address, slot, block_number);
        assert_eq(read[0], 13);
        assert_eq(read[1], 42);
    }

    #[test]
    fn test_storage_read() {
        let written = MockStruct { a: 13, b: 42 };

        let _ = OracleMock::mock("storageRead").returns(written.serialize());

        let read: MockStruct = storage_read(address, slot, block_number);
        assert_eq(read.a, 13);
        assert_eq(read.b, 42);
    }
}
