# Archiver

Archiver is a service which is used to fetch data on-chain data and present them in a nice-to-consume form.
The on-chain data specifically are the following events:

1. `L2BlockProcessed` event emitted on Rollup contract,
2. `ContractDeployment` event emitted on ContractDeploymentEmitter contract,
3. `MessageAdded` event emitted on Inbox contract,

The interfaces defining how the data can be consumed from the archiver are `L2BlockSource`, `L2LogsSource` and `ContractDataSource`.

## Usage

To install dependencies and build the package run `yarn install` followed by `yarn build`.
To run test execute `yarn test`.

To start the service export `ETHEREUM_HOST` (defaults to `http://127.0.0.1:8545/`), `ARCHIVER_POLLING_INTERVAL` (defaults to `1000 ms`), `ROLLUP_CONTRACT_ADDRESS`, `INBOX_CONTRACT_ADDRESS`, `CONTRACT_DEPLOYMENT_EMITTER_ADDRESS` environmental variables and start the service with `yarn start`.
