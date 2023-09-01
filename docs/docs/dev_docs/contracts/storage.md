# Storage

> A common convention in Noir Contracts is to declare the state variables for your Noir Contract in a `storage.nr` file inside your project (see [directory structure](./layout.md#directory-structure)). This is just a convention, though: your contract would still work if you declare storage in the `main.nr` file.

State variables must be declared inside a struct. (This enables us to declare types composed of nested generics in Noir - see [types](./types.md)).

We could define any kinds of state variables in the Storage struct:

#include_code storage-declaration /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/storage.nr rust

See [State Variables](./state_variables.md) for how to initialise them.

Using Storage in a contract is like using any other struct in Noir. First, import the struct into the contract's `main.nr` file:

#include_code storage-import /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

For each function that needs access to the storage, initialise the storage inside the function, and call the state variables in it:

#include_code storage-init /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust
