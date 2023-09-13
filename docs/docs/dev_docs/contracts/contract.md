---
title: Contract
---

# Contract

A contract is a collection of persistent [state variables](#state-variables), and [functions](#functions) which may modify these persistent states. Functions and states within a contract's scope are said to belong to that contract. A contract can only access and modify its own state. If a contract wishes to access or modify another contract's state, it must make a [call](#calling-functions) to an external function of that other contract. For anything to happen on the Aztec network, an external function of a contract needs to be called.

A contract may be declared and given a name using the `contract` keyword (see snippet below). By convention, contracts are named in `PascalCase`.

```rust title="contract keyword"
// highlight-next-line
contract MyContract {

    // Functions and state variables belonging to this contract are not shown here.

}
```


> A note for vanilla Noir devs: There is no [`main()`](https://noir-lang.org/getting_started/breakdown/#mainnr) function within a Noir `contract` scope. This is because more than one function of a contract may be called and proven as external (as opposed to inlined by the compiler).

## Structure of a contract

- Private state variables
- Public state variables
- Private functions
- Public functions
- Encrypted events
- Unencrypted events