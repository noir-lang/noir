use dep::protocol_types::traits::{Deserialize, Serialize};
use crate::oracle::storage::{storage_read, storage_write};

pub fn read<T, N>(storage_slot: Field) -> T where T: Deserialize<N> {
    T::deserialize(storage_read(storage_slot))
}

pub fn write<T, N>(storage_slot: Field, value: T) where T: Serialize<N> {
    storage_write(storage_slot, value.serialize());
}

// Ideally we'd do the following, but we cannot because of https://github.com/noir-lang/noir/issues/4633
// pub fn read_historical<T, N>(
//     storage_slot: Field,
//     context: PrivateContext
// ) -> T where T: Deserialize<N> {
//     let mut fields = [0; N];
//     for i in 0..N {
//         fields[i] = public_storage_historical_read(
//                 context,
//                 storage_slot + i as Field,
//                 context.this_address()
//             );
//     }
//     T::deserialize(fields)
// }

mod tests {
    use dep::std::test::OracleMock;
    use dep::protocol_types::traits::{Deserialize, Serialize};
    use crate::public_storage;

    struct TestStruct {
        a: Field,
        b: Field,
    }

    impl Deserialize<2> for TestStruct {
        fn deserialize(fields: [Field; 2]) -> TestStruct {
            TestStruct { a: fields[0], b: fields[1] }
        }
    }

    impl Serialize<2> for TestStruct {
        fn serialize(self) -> [Field; 2] {
            [self.a, self.b]
        }
    }

    #[test]
    fn test_read() {
        let slot = 7;
        let written = TestStruct { a: 13, b: 42 };

        OracleMock::mock("storageRead").with_params((slot, 2)).returns(written.serialize());

        let read: TestStruct = public_storage::read(slot);
        assert_eq(read.a, 13);
        assert_eq(read.b, 42);
    }

    #[test]
    fn test_write() {
        let slot = 7;
        let to_write = TestStruct { a: 13, b: 42 };

        let mock = OracleMock::mock("storageWrite").returns([0; 2]); // The return value is unused

        public_storage::write(slot, to_write);
        assert_eq(mock.get_last_params(), (slot, to_write.serialize()));
    }
}
