
        struct Array<T, let N: u32> {
            inner: [T; N],
        }

        impl<T, let N: u32> Array<T, N> {
            pub fn len(self) -> u32 {
                N as u32
            }
        }

        fn main(xs: [Field; 2]) {
            let ys = Array {
                inner: xs,
            };
            assert(ys.len() == 2);
        }
    