# Aztec Monorepo

All the packages that make up [Aztec](https://docs.aztec.network).

- [**`l1-contracts`**](/l1-contracts): Solidity code for the Ethereum contracts that process rollups
- [**`yarn-project`**](/yarn-project): Typescript code for client and backend
- [**`docs`**](/docs): Documentation source for the docs site

## Popular packages

- [Aztec.nr](./yarn-project/aztec-nr/): A [Noir](https://noir-lang.org) framework for smart contracts on Aztec.
- [Aztec](./yarn-project/aztec/): A package for starting up local dev net modules, including a local 'sandbox' devnet, an Ethereum network, deployed rollup contracts and Aztec execution environment.
- [Aztec.js](./yarn-project/aztec.js/): A tool for interacting with the Aztec network. It communicates via the [Private Execution Environment (PXE)](./yarn-project/pxe/).
- [Example contracts](./yarn-project/noir-contracts/): Example contracts for the Aztec network, written in Noir.
- [End to end tests](./yarn-project/end-to-end/): Integration tests written in Typescript--a good reference for how to use the packages for specific tasks.
- [Aztec Boxes](./boxes/): Example starter projects.

## Issues Board

All issues being worked on are tracked on the [Aztec Github Project](https://github.com/orgs/AztecProtocol/projects/22). For a higher-level roadmap, check the [milestones overview](https://docs.aztec.network/about_aztec/roadmap/main) section of our docs.

## Development Setup

Run `bootstrap.sh` in the project root to set up your environment. This will update git submodules, download ignition transcripts, install Foundry, compile Solidity contracts, install the current node version via nvm, and build all typescript packages.

Alternatively, to just hack on Noir contracts and Typescript, run `BOOTSTRAP_USE_REMOTE_CACHE=1 ./bootstrap.sh`, which will download existing builds for barretenberg and nargo from the CI cache. Note that this only works on Ubuntu.

To build Typescript code, make sure to have [`nvm`](https://github.com/nvm-sh/nvm) (node version manager) installed.

To build noir code, make sure that you are using the version from `yarn-project/noir-compiler/src/noir-version.json`.

Install nargo by running

```
noirup -v TAG_FROM_THE_FILE
```

## Continuous Integration

This repository uses CircleCI for continuous integration. Build steps are managed using [`build-system`](https://github.com/AztecProtocol/build-system). Small packages are built and tested as part of a docker build operation, while larger ones and end-to-end tests spin up a large AWS spot instance. Each successful build step creates a new docker image that gets tagged with the package name and commit.

All packages need to be included in the [build manifest](`build_manifest.json`), which declares what paths belong to each package, as well as dependencies between packages. When the CI runs, if none of the rebuild patterns or dependencies were changed, then the build step is skipped and the last successful image is re-tagged with the current commit. Read more on the [`build-system`](https://github.com/AztecProtocol/build-system) repository README.

It is faster to debug CI failures within a persistent ssh session compared to pushing and waiting. You can create a session with "Rerun step with SSH" on CircleCI which will generate an ssh command for debugging on a worker. Run that command locally and then do

```bash
cd project
./build-system/scripts/setup_env "$(git rev-parse HEAD)" "" "" ""
source /tmp/.bash_env*
{start testing your CI commands here}
```

This provide an interactive environment for debugging the CI test.

## Debugging

Logging goes through the [DebugLogger](yarn-project/foundation/src/log/debug.ts) module in Typescript. To see the log output, set a `DEBUG` environment variable to the name of the module you want to debug, to `aztec:*`, or to `*` to see all logs.

## Releases

Releases are driven by [release-please](https://github.com/googleapis/release-please), which maintains a 'Release PR' containing an updated CHANGELOG.md since the last release. Triggering a new release is simply a case of merging this PR to master. A [github workflow](./.github/workflows/release_please.yml) will create the tagged release triggering CircleCI to build and deploy the version at that tag.

## Contribute

There are many ways you can participate and help build high quality software. Check out the [contribution guide](CONTRIBUTING.md)!

## Syncing noir

We currently use [git-subrepo](https://github.com/ingydotnet/git-subrepo) to manage a mirror of noir. This tool was chosen because it makes code checkout and development as simple as possible (compared to submodules or subtrees), with the tradeoff of complexity around sync's.

There is an automatic mirror pushing noir to a PR on noir side. If the mirror is not working, generally we need to do a "git subrepo pull noir".

Recovering if the sync is not happening with basic pull commands:
 - manually editing the commit variable in noir/.gitrepo: 
   this needs to exist in the branch we push to, and have the same content as our base. This is similar to submodules, except instead of pointing to the final state of the module, it points to the last commit we have sync'd from, for purposes of commit replay. This can be fixed to match the commit in master after merges.
 - manually editing the parent variable in noir/.gitrepo: this is the parent of the last sync commit on aztec side. If you get errors with a commit not being found in the upstream repo, and the commit mentioned is not the commit variable above, it might indicate this is somehow incorrect. This can happen when commit content is ported without its history, e.g. squashes
 - use pull --force ONLY where you would use git reset. That is, if you really want to match some upstream noir for a purpose its fine, but you'll lose local changes (if any)
