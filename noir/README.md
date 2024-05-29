# Aztecs Build of Noir

We subrepo noir into the folder `noir-repo`.
This folder contains dockerfiles and scripts for performing our custom build of noir for the monorepo.

# Syncing with the main Noir repository

In order to keep aztec-packages in step with the main Noir repository we need to periodically sync between them.

Syncing from aztec-packages into noir currently attempts to revert any changes in Noir since the last sync so it's recommended to always sync from Noir first to ensure that aztec-packages is up-to-date.

## Syncing from Noir to aztec-packages.

To start the sync run [this action](https://github.com/AztecProtocol/aztec-packages/actions/workflows/pull-noir.yml) manually (click the "Run Workflow" button in the top right). aztec-bot will then open a new PR which does the initial sync, this will have merge conflicts with master which will need to be resolved.

## Syncing from aztec-packages to Noir.

When syncing from aztec-packages to Noir it's important to check that the latest release of `bb` uses the same ACIR serialization format as the current master commit. This is because Noir uses a released version of barretenberg rather than being developed in sync with it, it's then not possible to sync if there's been serialization changes since the last release.

To start the sync run [this action](https://github.com/AztecProtocol/aztec-packages/actions/workflows/mirror-noir-subrepo.yml) manually (click the "Run Workflow" button in the top right). aztec-bot will then open a new PR in the `noir-lang/noir` repository which does the initial sync, this will have merge conflicts with master which will need to be resolved.
