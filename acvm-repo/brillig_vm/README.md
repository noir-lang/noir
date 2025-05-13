# Brillig VM
The Brillig VM is a general purpose virtual machine that is designed to execute the [Brillig bytecode](../brillig/README.md).
It serves as the runtime environment for Brillig, which is a bytecode format used within the Noir ecosystem.

This VM provides a lightweight execution environment for running computational tasks that may be difficult or inefficient to express directly in circuit constraints. This enables Noir programs to perform complex computations outside of the constraint system while still being able to interact with constrained code.

The VM supports various operations including arithmetic operations over fields and integers, memory management, conditional execution, and foreign function calls that can interact with external systems. The VM processes opcodes sequentially, manages memory, and handles control flow, providing the foundation for executing Brillig bytecode generated from Noir programs.

## Documentation

For detailed documentation, visit <https://noir-lang.github.io/noir/docs/brillig_vm/index.html>.