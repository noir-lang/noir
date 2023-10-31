// This is here because x[index] = value is not working in Noir right now
// when x is &mut
fn noir_array_set<N>(x : &mut [Field; N], index : u64, value : Field) {
   // x[index] = value;
}

error: Expected type an array, found type &mut [Field; N]
   ┌─ lib.nr:95:15
   │
95 │    x[index] = value;
   │               -----
   │



// This is here because x[index] is not working in Noir right now
// when x is &mut 
fn noir_array_get<N>(x : &mut [Field; N], index : u64) -> Field {
   x[index]
}

error: Expected type Array, found type &mut [Field; N]
    ┌─ lib.nr:100:4
    │
100 │    x[index]
    │    -
    │
