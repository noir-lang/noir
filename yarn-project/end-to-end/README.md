# End to End

This package includes end-to-end tests that cover Aztec's main milestones.
These can be run locally either by starting anvil on a different terminal.

```
anvil -p 8545 --host 0.0.0.0 --chain-id 31337
```

and then running

```
yarn test
```

Or by running

```
yarn test:integration
```

which will spawn the two processes.

You can also run this by `docker-compose up` which will spawn 2 different containers for Anvil and the test runner.
