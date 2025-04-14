
        fn main(d: [Field; 2]) {
            if len(d) == 2 {              // use 1 of d
                if len(d) == 2 {          // use 2 of d
                    assert(eq(d, [5, 6]));  // use 3 of d
                }
            } else {
                assert(eq(d, [5, 6]));      // use 4 of d
            }
        }

        fn eq(lhs: [Field; 2], rhs: [Field; 2]) -> bool {
            (lhs[0] == rhs[0]) & (lhs[1] == rhs[1])
        }
        
        fn len(arr: [Field; 2]) -> u32 {
            2
        }
        