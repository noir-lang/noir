
        struct W<let N: u1> {}
        
        fn foo<let N: u1>(_x: W<N>) -> W<(0 * N) / (N % N)> {
            W {}
        }
        
        fn bar<let N: u1>(_x: W<N>) -> u1 {
            N
        }
        
        fn main() -> pub u1 {
            let w_0: W<0> = W {};
            let w: W<_> = foo(w_0);
            bar(w)
        }
    