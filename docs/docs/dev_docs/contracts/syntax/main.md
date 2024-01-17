import DocCardList from '@theme/DocCardList';
import { AztecPackagesVersion } from "@site/src/components/Version";

# Aztec.nr Syntax

[Noir](https://noir-lang.org/) is a language which is agnostic to proof systems and use cases. Rather than baking Aztec-specific keywords and smart contract types directly into Noir (which would break this agnosticism), we have developed a framework -- written in Noir -- whose types and methods provide rich smart contract semantics.

On top of [Noir's stdlib](https://noir-lang.org/docs/noir/standard_library/cryptographic_primitives), we provide [Aztec.nr](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/aztec-nr) for writing contracts on Aztec.

Aztec.nr contains abstractions which remove the need to understand the low-level Aztec protocol. Notably, it provides:

- Public and private [state variable types](./storage/main.md)
- Some pre-designed notes
- Functions for [emitting](./events.md) encrypted and unencrypted logs
- [Oracle functions](./functions.md#oracle-functions) for accessing:
  - private state
  - secrets
- Functions for communicating with [Ethereum L1](../portals/main.md)

To setup a aztec-nr project, follow the [setup instructions](../setup.md)

<DocCardList />
