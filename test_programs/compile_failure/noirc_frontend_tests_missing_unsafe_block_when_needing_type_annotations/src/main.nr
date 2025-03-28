
    fn main() {
        let z = BigNum { limbs: [2, 0, 0] };
        assert(z.__is_zero() == false);
    }

    struct BigNum<let N: u32> {
        limbs: [u64; N],
    }

    impl<let N: u32> BigNum<N> {
        unconstrained fn __is_zero_impl(self) -> bool {
            let mut result: bool = true;
            for i in 0..N {
                result = result & (self.limbs[i] == 0);
            }
            result
        }
    }

    trait BigNumTrait {
        fn __is_zero(self) -> bool;
    }

    impl<let N: u32> BigNumTrait for BigNum<N> {
        fn __is_zero(self) -> bool {
            self.__is_zero_impl()
        }
    }
    