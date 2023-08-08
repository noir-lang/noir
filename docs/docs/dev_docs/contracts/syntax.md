# Noir Contract Syntax

[Vanilla Noir](https://noir-lang.org/) is a language which is agnostic to proof systems and use cases. Rather than baking Aztec-specific keywords and smart contract types directly into Noir (which would break this agnosticism), we have developed a library -- written in Vanilla Noir -- whose types and methods provide rich smart contract semantics.

## Aztec stdlib

On top of ['Vanialla Noir's' stdlib](https://noir-lang.org/standard_library/array_methods), we provide an  [Aztec stdlib](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-libs) for writing Noir Contracts. The Aztec stdlib contains abstractions which remove the need to understand the low-level Aztec protocol. Notably, it provides:
- Public and private [state variable types](./types.md)
- Ready-made notes.
- Functions for [emitting](./events.md) encrypted and unencrypted logs
- [Oracle functions](./functions.md#oracle-calls) for accessing:
  - private state
  - secrets
- Functions for communicating with Ethereum L1


To import the Aztec stdlib into your Noir Contract project, simply include it as a dependency:

:::danger TODO
https://github.com/AztecProtocol/aztec-packages/issues/1335
:::

#include_code importing-aztec /yarn-project/noir-contracts/src/contracts/private_token_contract/Nargo.toml toml