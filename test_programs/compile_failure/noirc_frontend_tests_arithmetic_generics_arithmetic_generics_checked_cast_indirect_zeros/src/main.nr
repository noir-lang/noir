
        struct W<let N: Field> {}
        
        fn foo<let N: Field>(_x: W<N>) -> W<(N - N) % (N - N)> {
            W {}
        }
        
        fn bar<let N: Field>(_x: W<N>) -> Field {
            N
        }
        
        fn main() {
            let w_0: W<0> = W {};
            let w = foo(w_0);
            let _ = bar(w);
        }
    