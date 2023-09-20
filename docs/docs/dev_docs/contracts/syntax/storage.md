# Storage

In an Aztec.nr contract, storage is to be defined as a single struct. (This enables us to declare types composed of nested generics in Noir).

The struct **must** be called `Storage` for the Aztec.nr library to properly handle it (will be fixed in the future to support more flexibility).
An example of such a struct could be as follows:

#include_code storage-struct-declaration /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

:::info
If your storage includes private state variables it must include a `compute_note_hash_and_nullifier` function to allow the RPC to process encrypted events, see [encrypted events](./events.md#processing-encrypted-events) for more.
:::

In the storage stuct, we set up a mixture of public and private state variables. The public state variables can be read by anyone, and functions manipulating them are executed by the sequencer, (see [functions](./functions.md#public-functions)). Private state variables are only readable by their owner, or people whom the owner has shared the data with. 

As mentioned earlier in the foundational concepts ([state model](./../../../concepts/foundation/state_model.md) and [private/public execution](./../../../concepts/foundation/communication/public_private_calls.md)) private state follows a UTXO model. Where note pre-images are only known to those able to decrypt them.

It is currently required to specify the length of the types when declaring the storage struct. If your type is a struct, this will be the number of values in your struct ( with arrays flattened ).

Since Aztec.nr is a library written in Noir, we can use the types defined in Noir, so it can be useful to consult the [Noir documentation](https://noir-lang.org/language_concepts/data_types) for information on types.

Currently, the sandbox also requires that you specify how this storage struct is "initialized". This is done by specifying an `init` function that is run in functions that rely on reading or altering the state variables. 

An example of such a function for the above storage struct would be:

#include_code storage-declaration /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

:::warning Using slot `0` is not supported!
No storage values should be initialized at slot `0`. If you are using slot `0` for storage, you will not get an error when compiling, but the contract will not be updating the storage! This is a known issue that will be fixed in the future.
:::

In [State Variables](./state_variables.md) we will see in more detail what each of these types are, how they work and how to initialize them.

To use storage in functions, e.g., functions where you would read or write storage, you need to initialize the struct first, and then you can read and write afterwards.

#include_code storage-init /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

:::info
https://github.com/AztecProtocol/aztec-packages/pull/2406 is removing the need to explicitly initialize the storage in each function before reading or writing. This will be updated in the docs once the PR is merged.
:::