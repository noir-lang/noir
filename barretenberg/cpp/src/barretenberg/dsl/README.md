# Domain Specific Language

This package adds support to use [ACIR](https://github.com/noir-lang/noir/tree/master/acvm-repo/acir) with barretenberg. 

## Serialization Changes

There are two types of breaking serialization changes. One that alters that alters the internal ACIR structure passed to barretenberg, and one that changes how we serialize the buffer passed to barretenberg.

1. Internal Structure Change

Go to the ACVM `acir` crate and re-run the [serde reflection test](../../../../../noir/noir-repo/acvm-repo/acir/src/lib.rs#L51). Remember to comment out the hash check to write the updated C++ serde file. Copy that file into the `dsl` package's [serde folder](./acir_format/serde/) where you will see an `acir.hpp` file. 

You will have to update a couple things in the new `acir.hpp`:
- Replace all `throw serde::deserialization_error` with `throw_or_abort`
- The top-level struct (such as `Program`) will still use its own namespace for its internal fields. This extra `Program::` can be removed from the top-level `struct Program { .. }` object.

The same can then be done for any breaking changes introduced to `witness_stack.hpp`.

2. Full Breaking Change

This type of breaking change is rarely expected to happen, however, due to its nature there are multiple consumers of the pre-existing serialization you should be aware of if you ever need to make this change.

A full change is when you attempt to change the object whose buffer we are actually deserializing in barretenberg. To give more detail, when [deserializing the constraint buffer](./acir_format/acir_to_constraint_buf.hpp#366) if the object for which we call `bincodeDeserialize` on changes, anything that has pre-existing ACIR using the previous object's `bincodeDeserialize` method will now fail. The serialization is once again determined by the top-level object in the [acir crate](../../../../../noir/noir-repo/acvm-repo/acir/src/circuit/mod.rs). After performing the steps outlined for an internal structure breaking change as listed in (1) above, we need to update all consumers of barretenberg.

Even if you correctly update all serialization in [acvm_js](../../../../../noir/noir-repo/acvm-repo/acvm_js/README.md) such as during [execution](../../../../../noir/noir-repo/acvm-repo/acvm_js/src/execute.rs#57), there is multiple places the `yarn-project` uses the ACIR top-level serialization. The `yarn-project` sequencer also uses the native `acvm_cli tool` that has an execute method that [expects raw byte code](../../../../../noir/noir-repo/tooling/acvm_cli/src/cli/execute_cmd.rs#63). 

In the context of Aztec we need to regenerate all the artifacts in [noir-projects](../../../../../noir-projects/bootstrap.sh). This regeneration assumes that we have rebuilt the compilers (Noir compiler and AVM transpiler) to use the new serialization. After regenerating these artifacts we can bootstrap the yarn-project. There are multiple packages in the yarn-project that rely on pre-computed artifacts such as `yarn-project/circuits.js` and `yarn-project/protocol-contracts`. 

The Aztec artifacts can be individually regenerated as well using `yarn test -u`. 
You can also run the command on the relevant workspaces, which at the time are the following:
```
yarn workspace @aztec/circuits.js test -u
yarn workspace @aztec/protocol-contracts test -u
```
