use crate::cmp::Eq;

unconstrained fn __get_shuffle_indices<T, let N: u32>(lhs: [T; N], rhs: [T; N]) -> [Field; N]
where
    T: Eq,
{
    let mut shuffle_indices: [Field; N] = [0; N];

    let mut shuffle_mask: [bool; N] = [false; N];
    for i in 0..N {
        let mut found = false;
        for j in 0..N {
            if ((shuffle_mask[j] == false) & (!found)) {
                if (lhs[i] == rhs[j]) {
                    found = true;
                    shuffle_indices[i] = j as Field;
                    shuffle_mask[j] = true;
                }
            }
            if (found) {
                continue;
            }
        }
        assert(found == true, "check_shuffle, lhs and rhs arrays do not contain equivalent values");
    }

    shuffle_indices
}

unconstrained fn __get_index<let N: u32>(indices: [Field; N], idx: Field) -> Field {
    let mut result = 0;
    for i in 0..N {
        if (indices[i] == idx) {
            result = i as Field;
            break;
        }
    }
    result
}

pub(crate) fn check_shuffle<T, let N: u32>(lhs: [T; N], rhs: [T; N])
where
    T: Eq,
{
    /// Safety: shuffle_indices is ensured to be a permutation of 0..N, and then
    /// shuffle_indices is ensured to map lhs to rhs: assert(lhs[i] == rhs[shuffle_indices[i]]), for all i in 0..N
    unsafe {
        let shuffle_indices = __get_shuffle_indices(lhs, rhs);

        for i in 0..N {
            let idx = __get_index(shuffle_indices, i as Field);
            assert_eq(shuffle_indices[idx], i as Field);
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
}
