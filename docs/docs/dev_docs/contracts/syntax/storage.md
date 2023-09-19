# Storage

State variables must be declared inside a struct. (This enables us to declare types composed of nested generics in Noir).

We could define any kinds of state variables in the Storage struct:

#include_code storage-declaration /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust

See [State Variables](./state_variables.md) for how to initialise them.

Using Storage in a contract is like using any other struct in Noir. For each function that needs access to the storage, initialise the storage inside the function, and then access its state variable members:

#include_code storage-init /yarn-project/noir-contracts/src/contracts/docs_example_contract/src/main.nr rust
