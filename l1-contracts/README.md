# L1 Contracts

This directory contains the Ethereum smart contracts for progressing the state of the Rollup.

## Installation

You can install foundry as https://book.getfoundry.sh/ or by running the `./bootstrap.sh` script.

Alternatively you can use docker instead, it will handle installations and run tests. Simply run `docker build .` from the `l1-contracts` folder.

## Structure

The `src` folder contain contracts that is to be used by the local developer testnet. It is grouped into 3 catagories:

- `core` contains the required contracts, the bare minimum
- `mock` contains stubs, for now an always true verifier.
- `periphery` stuff that is nice to have, convenience contracts and functions belong in here.

## Running tests

The tests are located in the `test` folder, and execute two consecutive L2Blocks. The blocks and the values they are checked against is generated using the block builder tests (there also is a typescript test in `l2-block-publisher.test.ts` that tests E2E). The tests are currently limited in functionality as it is mainly decoding happening, but will expand over time to include L1 <-> L2 communication and cross chain applications.

As mentioned earlier, you can also use docker. If you rerun `docker build .` after changing the contracts, it will use a cache for most values, and rerun your tests in a few seconds.

## Formatting

We use `forge fmt` to format. But follow a few general guidelines beyond the standard:

- use `_` prefix for function arguments, e.g.,
  - Don't `function transfer(address to, uint256 amount);`
  - Do `function transfer(address _to, uint256 _amount);`
- use `_` prefix for `internal` and `private` functions.

## Contracts:

The contracts are in a very early stage, and don't worry about gas costs right now. Instead they prioritize development velocity.

### Decoder

Job: Extract values from `L2Block`

The decoder is a core rollup contract which takes the `L2Block` bytes and computes/extracts values required to:

1. keep track of the state
1. perform proof verification

If the structure of the `L2Block` changes, so should the decoder!

### Rollup

Job: Keep track of state and perform state transitions.

It is the job of the rollup contract to store the state of the rollup and progress it when receiving a new L2 block that is built on top of the current state.

Currently not running any proofs _nor_ access control so blocks can be submitted by anyone and can be complete garbage.

### ContractDeploymentEmitter

Job: Share Contract Deployment public data on chain.

For now, this include bytecode for contract deployment, but over time this will be verified for public functions.

---

# Linter

We use an extended version of solhint (https://github.com/LHerskind/solhint) to include custom rules. These custom rules relate to how errors should be named, using custom errors instead of reverts etc, see `.solhint.json` for more specifics about the rules.

The linter is the only node module we need which is the reason behind not going full yarn-project on it.

To run the linter, simply run:

```bash
yarn lint
```

---

# Slither

We use slither as an automatic way to find blunders and common vulnerabilities in our contracts. It is not part of the docker image due to its slowness, but it can be run using the following command to generate a markdown file with the results:
```bash
yarn slither
```

When this command is run in CI, it will fail if the markdown file generated in docker don't match the one in the repository. 

We assume that you already have slither installed. You can install it with `pip3 install slither-analyzer`. It is kept out of the bootstrap script as it is not a requirement for people who just want to run tests or are uninterested in the contracts.

> We are not running the `naming-convention` detector because we have our own rules for naming which is enforced by the linter.