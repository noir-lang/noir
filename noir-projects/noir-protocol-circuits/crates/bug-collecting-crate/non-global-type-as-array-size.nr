    let num_bytes : Field = 64;

    let input = [0;num_bytes];


error: Cannot find a global or generic type parameter named `plain::num_bytes`
    ┌─ hash.nr:163:20
    │
163 │     let input = [0;num_bytes];
    │                    --------- Only globals or generic type parameters are allowed to be used as an array type's length
    │