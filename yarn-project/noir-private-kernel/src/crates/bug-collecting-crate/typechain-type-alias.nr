The typescript binding generator has a bug when we use type aliases because 
the abi says that they have the same struct path.

For example:



struct Generic<N> {
    x : [Field; N]
}

struct Concrete {
    gen2 : Generic<2>,
    gen4 : Generic<4>,

}

fn main(input: Concrete) -> pub Field {
    0
}


The following code will generate the json:

{"hash":17271335012890464242,"backend":"acvm-backend-barretenberg","abi":{"parameters":[{"name":"input","type":{"kind":"struct","path":"Concrete","fields":[{"name":"gen2","type":{"kind":"struct","path":"Generic","fields":[{"name":"x","type":{"kind":"array","length":2,"type":{"kind":"field"}}}]}},{"name":"gen4","type":{"kind":"struct","path":"Generic","fields":[{"name":"x","type":{"kind":"array","length":4,"type":{"kind":"field"}}}]}}]},"visibility":"private"}],"param_witnesses":{"input":[1,2,3,4,5,6]},"return_type":{"kind":"field"},"return_witnesses":[7]},"bytecode":"H4sIAAAAAAAA/6WPSwqAMAxE69/jJE3SJjuvYrG9/xEsqFDQnQ8egVmEmdU517k3T7bdlyAw5+gzEu7gLakASwqKiqJyeCXKyhotWQRDpoxFjApcLM0v+MncdOyrQ3WsTtX5Y8PSZCeMnX6J8AAAAA=="}

And subsequently generate this typescript file:

export type FixedLengthArray<T, L extends number> = L extends 0 ? never[]: T[] & { length: L }

export type Field = string;

export interface Generic {
  x: FixedLengthArray<Field, 2>;
}



export interface Concrete {
  gen2: Generic;
  gen4: Generic;
}

export interface ReturnType {
  value: Field;
}

export interface InputType {
  input: Concrete;
}

----

The important thing to notice is that there is one Generic and it gets instantiated with
the length of the first parameter.

We can go two ways with this, either we end up with something like:

export interface Generic<N extends number> {
  x: FixedLengthArray<Field, N>;
}

export interface Concrete {
  gen2: Generic<2>;
  gen4: Generic<4>;
}

or we do something like:

export interface Generic2 {
  x: FixedLengthArray<Field, 2>;
}
export interface Generic4 {
  x: FixedLengthArray<Field, 2>;
}

export interface Concrete {
  gen2: Generic2;
  gen4: Generic4;
}

First seems to have better devex and less copy pasting but requires more complicated code.
Perhaps this can be aided by the compiler, if we save this information before monomorphization
