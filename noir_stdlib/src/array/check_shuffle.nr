use crate::cmp::Eq;

unconstrained fn __get_shuffle_indices<T, let N: u32>(lhs: [T; N], rhs: [T; N]) -> [u32; N]
where
    T: Eq,
{
    let mut shuffle_indices: [u32; N] = [0; N];

    let mut shuffle_mask: [bool; N] = [false; N];
    for i in 0..N {
        let mut found = false;
        for j in 0..N {
            if (shuffle_mask[j] == false) {
                if (lhs[i] == rhs[j]) {
                    found = true;
                    shuffle_indices[i] = j;
                    shuffle_mask[j] = true;
                    break;
                }
            }
        }
        assert(found == true, "check_shuffle, lhs and rhs arrays do not contain equivalent values");
    }

    shuffle_indices
}

unconstrained fn __get_index<let N: u32>(indices: [u32; N], idx: u32) -> u32 {
    let mut result = 0;
    for i in 0..N {
        if (indices[i] == idx) {
            result = i;
            break;
        }
    }
    result
}

pub(crate) fn check_shuffle<T, let N: u32>(lhs: [T; N], rhs: [T; N])
where
    T: Eq,
{
    // Safety: shuffle_indices is ensured to be a permutation of 0..N, and then
    // shuffle_indices is ensured to map lhs to rhs: assert(lhs[i] == rhs[shuffle_indices[i]]), for all i in 0..N
    unsafe {
        let shuffle_indices = __get_shuffle_indices(lhs, rhs);

        for i in 0..N {
            let idx = __get_index(shuffle_indices, i);
            assert_eq(shuffle_indices[idx], i);
        }
        for i in 0..N {
            let idx = shuffle_indices[i];
            let expected = rhs[idx];
            let result = lhs[i];
            assert_eq(expected, result);
        }
    }
}

mod test {
    use crate::cmp::Eq;
    use super::check_shuffle;

    struct CompoundStruct {
        a: bool,
        b: Field,
        c: u64,
    }
    impl Eq for CompoundStruct {
        fn eq(self, other: Self) -> bool {
            (self.a == other.a) & (self.b == other.b) & (self.c == other.c)
        }
    }

    #[test]
    fn test_shuffle() {
        let lhs: [Field; 5] = [0, 1, 2, 3, 4];
        let rhs: [Field; 5] = [2, 0, 3, 1, 4];
        check_shuffle(lhs, rhs);
    }

    #[test]
    fn test_shuffle_identity() {
        let lhs: [Field; 5] = [0, 1, 2, 3, 4];
        let rhs: [Field; 5] = [0, 1, 2, 3, 4];
        check_shuffle(lhs, rhs);
    }

    #[test(should_fail_with = "check_shuffle, lhs and rhs arrays do not contain equivalent values")]
    fn test_shuffle_fail() {
        let lhs: [Field; 5] = [0, 1, 2, 3, 4];
        let rhs: [Field; 5] = [0, 1, 2, 3, 5];
        check_shuffle(lhs, rhs);
    }

    #[test(should_fail_with = "check_shuffle, lhs and rhs arrays do not contain equivalent values")]
    fn test_shuffle_duplicates() {
        let lhs: [Field; 5] = [0, 1, 2, 3, 4];
        let rhs: [Field; 5] = [0, 1, 2, 3, 3];
        check_shuffle(lhs, rhs);
    }

    #[test]
    fn test_shuffle_compound_struct() {
        let lhs: [CompoundStruct; 5] = [
            CompoundStruct { a: false, b: 0, c: 12345 },
            CompoundStruct { a: false, b: -100, c: 54321 },
            CompoundStruct { a: true, b: 5, c: 0xffffffffffffffff },
            CompoundStruct { a: true, b: 9814, c: 0xeeffee0011001133 },
            CompoundStruct { a: false, b: 0x155, c: 0 },
        ];
        let rhs: [CompoundStruct; 5] = [
            CompoundStruct { a: false, b: 0x155, c: 0 },
            CompoundStruct { a: false, b: 0, c: 12345 },
            CompoundStruct { a: false, b: -100, c: 54321 },
            CompoundStruct { a: true, b: 9814, c: 0xeeffee0011001133 },
            CompoundStruct { a: true, b: 5, c: 0xffffffffffffffff },
        ];
        check_shuffle(lhs, rhs);
    }

    #[test]
    fn test_shuffle_empty_array() {
        let lhs: [Field; 0] = [];
        let rhs: [Field; 0] = [];
        check_shuffle(lhs, rhs);
    }

    #[test]
    fn test_shuffle_single_element() {
        let lhs: [Field; 1] = [42];
        let rhs: [Field; 1] = [42];
        check_shuffle(lhs, rhs);
    }

    // arrays with duplicate values
    #[test]
    fn test_shuffle_with_duplicates() {
        let lhs: [Field; 4] = [1, 2, 1, 3];
        let rhs: [Field; 4] = [1, 3, 1, 2];
        check_shuffle(lhs, rhs);
    }

    #[test]
    fn test_shuffle_all_same_values() {
        let lhs: [Field; 5] = [42, 42, 42, 42, 42];
        let rhs: [Field; 5] = [42, 42, 42, 42, 42];
        check_shuffle(lhs, rhs);
    }

    #[test]
    fn test_shuffle_bool() {
        let lhs: [bool; 4] = [true, false, true, false];
        let rhs: [bool; 4] = [false, true, false, true];
        check_shuffle(lhs, rhs);
    }

    #[test]
    fn test_shuffle_compound_struct_with_duplicates() {
        let struct1 = CompoundStruct { a: true, b: 42, c: 100 };
        let struct2 = CompoundStruct { a: false, b: 0, c: 200 };
        let lhs: [CompoundStruct; 4] = [struct1, struct2, struct1, struct2];
        let rhs: [CompoundStruct; 4] = [struct2, struct1, struct2, struct1];
        check_shuffle(lhs, rhs);
    }

    #[test(should_fail_with = "check_shuffle, lhs and rhs arrays do not contain equivalent values")]
    fn test_shuffle_compound_struct_fail() {
        let lhs: [CompoundStruct; 2] =
            [CompoundStruct { a: true, b: 42, c: 100 }, CompoundStruct { a: false, b: 0, c: 200 }];
        let rhs: [CompoundStruct; 2] = [
            CompoundStruct { a: true, b: 42, c: 100 },
            CompoundStruct { a: false, b: 1, c: 200 }, // Different b field
        ];
        check_shuffle(lhs, rhs);
    }

    // test with negative field values
    #[test]
    fn test_shuffle_negative_values() {
        let lhs: [Field; 4] = [-1, -2, -3, -4];
        let rhs: [Field; 4] = [-4, -3, -2, -1];
        check_shuffle(lhs, rhs);
    }
}
