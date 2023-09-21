import DocCardList from '@theme/DocCardList';

# Aztec.nr Syntax

[Noir](https://noir-lang.org/) is a language which is agnostic to proof systems and use cases. Rather than baking Aztec-specific keywords and smart contract types directly into Noir (which would break this agnosticism), we have developed a framework -- written in Noir -- whose types and methods provide rich smart contract semantics.

On top of [Noir's stdlib](https://noir-lang.org/standard_library/array_methods), we provide [Aztec.nr](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/aztec-nr) for writing contracts on Aztec.

Aztec.nr contains abstractions which remove the need to understand the low-level Aztec protocol. Notably, it provides:

- Public and private [state variable types](./state_variables.md)
- Some pre-designed notes
- Functions for [emitting](./events.md) encrypted and unencrypted logs
- [Oracle functions](./functions.md#oracle-functions) for accessing:
  - private state
  - secrets
- Functions for communicating with [Ethereum L1](../portals/main.md)

To import Aztec.nr into your Aztec contract project, simply include it as a dependency. For example:

import { AztecPackagesVersion } from "@site/src/components/Version";

<CodeBlock language="toml">{`[package]
name = "token_contract"
authors = [""]
compiler_version = "0.1"
type = "contract"
 
[dependencies]
# Framework import
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="${AztecPackagesVersion()}", directory="yarn-project/aztec-nr/aztec" }
 
# Utility dependencies
value_note = { git="https://github.com/AztecProtocol/aztec-packages/", tag="${AztecPackagesVersion()}", directory="yarn-project/aztec-nr/value-note"}
safe_math = { git="https://github.com/AztecProtocol/aztec-packages/", tag="${AztecPackagesVersion()}", directory="yarn-project/aztec-nr/safe-math"}
`}</CodeBlock>

:::info 
Note: currently the dependency name ***MUST*** be `aztec`. The framework expects this namespace to be available when compiling into contracts. This limitation may be removed in the future.
:::

<DocCardList />
