# Aztec Monorepo

All the packages that make up [Aztec](https://docs.aztec.network/what-is-aztec).

- [**`circuits`**](/circuits): C++ code for circuits and cryptographic functions
- [**`l1-contracts`**](/l1-contracts): Solidity code for the Ethereum contracts that process rollups
- [**`yarn-project`**](/yarn-project): Typescript code for client and backend

## Issues Board

All issues being worked on are tracked on the [Aztec Github Project](https://github.com/orgs/AztecProtocol/projects/22). For a higher-level roadmap, check the [milestones overview](https://docs.aztec.network/aztec/milestones) section of our docs.

## Development Setup

Run `bootstrap.sh` in the project root to set up your environment. This will update git submodules, download ignition transcripts, build all C++ circuits code, install Foundry, compile Solidity contracts, install the current node version via nvm, and build all typescript packages.

To build the C++ code, follow the [instructions in the circuits subdirectory](./circuits/README.md), which contains all of the ZK circuit logic for Aztec. Note that "barretenberg", Aztec's underlying cryptographic library, can be found inside the circuits subdirectory as well and is automatically built as a side effect of building the circuits.

To build Typescript code, make sure to have [`nvm`](https://github.com/nvm-sh/nvm) (node version manager) installed.

## Continuous Integration

This repository uses CircleCI for continuous integration. Build steps are managed using [`build-system`](https://github.com/AztecProtocol/build-system). Small packages are built and tested as part of a docker build operation, while larger ones and end-to-end tests spin up a large AWS spot instance. Each successful build step creates a new docker image that gets tagged with the package name and commit.

All packages need to be included in the [build manifest](`build_manifest.json`), which declares what paths belong to each package, as well as dependencies between packages. When the CI runs, if none of the rebuild patterns or dependencies were changed, then the build step is skipped and the last successful image is re-tagged with the current commit. Read more on the [`build-system`](https://github.com/AztecProtocol/build-system) repository README.

## Debugging

Logging goes through the [`info` and `debug`](circuits/cpp/barretenberg/cpp/src/barretenberg/common/log.hpp) functions in C++, and through the [DebugLogger](yarn-project/foundation/src/log/debug.ts) module in Typescript. To see the log output, set a `DEBUG` environment variable to the name of the module you want to debug, to `aztec:*`, or to `*` to see all logs.


## Contribute

There are many ways you can participate and help build high quality software. Check out the [contribution guide](CONTRIBUTING.md)!
