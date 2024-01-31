---
title: Constructor
---
This page talks about declaring constructors in functions. 

For a more hands-on guide into declaring a constructor, follow the [private voting tutorial](../../../tutorials/writing_private_voting_contract.md).

A special `constructor` function **must** be declared within a contract's scope.
- A constructor doesn't have a name, because its purpose is clear: to initialize contract state.
- In Aztec terminology, a constructor is always a '`private` function' (i.e. it cannot be a `public` function).
- A constructor behaves almost identically to any other function. It is just important for Aztec to be able to identify this function as special: it may only be called once, and will not be deployed as part of the contract.

Although you can have a constructor that does nothing, you might want to do something with it, such as setting the deployer as an owner.

#include_code constructor /yarn-project/noir-contracts/contracts/escrow_contract/src/main.nr rust
