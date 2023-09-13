# BB

## Why is this needed?

Barretenberg is a library that allows one to create and verify proofs. One way to specify the circuit that one will use to create and verify
proofs over is to use the Barretenberg standard library. Another way, which pertains to this module is to supply the circuit description using 
an IR called [ACIR](https://github.com/noir-lang/acvm). This binary will take as input ACIR and witness values described in the IR to create proofs.


## FilePath vs Stdout

For commands which allow you to send the output to a file using `-o {filePath}`, there is also the option to send the output to stdout by using `-o -`.

## Maximum Circuit Size

Currently the binary downloads an SRS that can be used to prove the maximum circuit size. This maximum circuit size parameter is a constant in the code and has been set to $2^{23}$ as of writing. This maximum circuit size differs from the maximum circuit size that one can prove in the browser, due to WASM limits.