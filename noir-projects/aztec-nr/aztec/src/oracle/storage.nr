use dep::protocol_types::traits::Deserialize;

#[oracle(storageRead)]
unconstrained fn storage_read_oracle<N>(storage_slot: Field, length: Field) -> [Field; N] {}

unconstrained pub fn raw_storage_read<N>(storage_slot: Field) -> [Field; N] {
    storage_read_oracle(storage_slot, N)
}

unconstrained pub fn storage_read<T, N>(storage_slot: Field) -> T where T: Deserialize<N> {
    T::deserialize(raw_storage_read(storage_slot))
}

mod tests {
    use crate::oracle::storage::{raw_storage_read, storage_read};

    use std::test::OracleMock;
    use crate::test::mocks::mock_struct::MockStruct;

    #[test]
    fn test_raw_storage_read() {
        let slot = 7;
        let written = MockStruct { a: 13, b: 42 };

        let _ = OracleMock::mock("storageRead").with_params((slot, 2)).returns(written.serialize());

        let read: [Field; 2] = raw_storage_read(slot);
        assert_eq(read[0], 13);
        assert_eq(read[1], 42);
    }

    #[test]
    fn test_storage_read() {
        let slot = 7;
        let written = MockStruct { a: 13, b: 42 };

        let _ = OracleMock::mock("storageRead").with_params((slot, 2)).returns(written.serialize());

        let read: MockStruct = storage_read(slot);
        assert_eq(read.a, 13);
        assert_eq(read.b, 42);
    }
}
