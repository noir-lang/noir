# aztec3::circuits::abis

Contains all ABIs for use by:
- Test app circuits
- Kernel circuits
- Rollup circuits

All ABIs are generalised by an `NCT` template parameter (meaning `NativeOrCircuitTypes`). `NCT` can be either `plonk::stdlib::types::NativeTypes` or `plonk::stdlib::types::CircuitTypes<Composer>` for some `Composer`. The idea being, there's a single implementation of every struct/class for native and circuit types. NativeType structs can be switched to CircuitType with the `to_circuit_type()` method.