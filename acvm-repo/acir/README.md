### Brillig

This opcode is used as a hint for the solver when executing (solving) the
circuit. The opcode does not generate any constraint and is usually the result
of the compilation of an unconstrained noir function.

Let's see an example with euclidean division.
The normal way to compute `a/b`, where `a` and `b` are 8-bits integers, is to
implement the Euclidean algorithm which computes in a loop (or recursively)
modulus of the kind 'a mod b'. Doing this computation requires a lot of steps to
be properly implemented in ACIR, especially the loop with a condition. However,
euclidean division can be easily constrained with one assert-zero opcode:
`a = bq+r`, assuming `q` is 8 bits and `r<b`. Since these assumptions can easily
written with a few range opcodes, euclidean division can in fact be implemented
with a small number of opcodes.

However, in order to write these opcodes we need the result of the division
which are the witness `q` and `r`. But from the constraint `a=bq+r`, how can the
solver figure out how to solve `q` and `r` with only one equation? This is where
brillig/unconstrained function come into action. We simply define a function that
performs the usual Euclid algorithm to compute `q` and `r` from `a` and `b`.
Since Brillig opcode does not generate constraint, it won't be provided to the
proving system but simply used by the solver to compute the values of `q` and
`r`.

In summary, solving a Brillig opcode performs the computation defined by its
bytecode, on the provided inputs, and assign the result to the outputs witnesses,
without adding any constraint.

NOTE: see the [circuit/opcodes.rs](src/circuit/opcodes.rs) file for the most
up-to-date documentation on these opcodes.
