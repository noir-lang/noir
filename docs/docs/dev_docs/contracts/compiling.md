# Compiling contracts

Please use the [TUTORIAL-TEMPLATE](../../TUTORIAL_TEMPLATE.md) for standalone guides / tutorials.

:::danger TODO
TODO: this entire page
:::
## Compiling a Noir Contract

You can use the `master` branch of Noir/nargo (hooray).

`cd path/to/src`

`nargo compile --contracts`

> Note: the output abi json artifact file needs some manual tweaking, currently, to insert a mocked verification key. See [this script](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/scripts/compile.sh) (which itself calls upon the two other scripts in that file) to see how compilation and injection of the verification key is being done currently. [noir-compiler](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-compiler) is intended to replace these scripts, but ideally nargo should be outputting aztec-contract-compatible abis.

https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts

## Noir Contract Artifacts

Compiling a Noir Contract will output an artifact -- a JSON ABI containing information about the contract. This file can be read by `aztec.js`, which is a neat TypeScript wrapper which enables developers to easily deploy contracts, view functions, and send transactions.