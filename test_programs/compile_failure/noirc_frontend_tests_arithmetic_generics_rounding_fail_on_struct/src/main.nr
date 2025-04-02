
        struct W<let N: u32> {}

        fn foo<let N: u32, let M: u32>(_x: W<N>, _y: W<M>) -> W<N / M * M> {
            W {}
        }

        fn main() {
            let w_2: W<2> = W {};
            let w_3: W<3> = W {};
            // Do not simplify N/M*M to just N
            // This should be 3/2*2 = 2, not 3
            let _: W<3> = foo(w_3, w_2);
        }
    