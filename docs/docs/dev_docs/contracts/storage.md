# Storage

> A common convention in Noir Contracts is to declare the state variables for your Noir Contract in a `storage.nr` file inside your project (see [directory structure](./layout.md#directory-structure)). This is just a convention, though: your contract would still work if you declare storage in the `main.nr` file.

State variables must be declared inside a struct. (This enables us to declare types composed of nested generics in Noir - see [types](./types.md)).

By way of example, we could define a private state variable `balances`, mapping user addresses to their token balances:

#include_code storage-declaration /yarn-project/noir-contracts/src/contracts/private_token_contract/src/storage.nr rust

#include_code storage-import /yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr rust

State variables come in two flavours: **public** state and **private** state. <INSERT LINK TO DOC EXPLAINING PRIVATE STATE & UTXOS>.