# Canary

This package is designed for running a comprehensive end to end test of the system using deployed artifacts. It is built and executed after the deployment of artifacts to npm and dockerhub.

## Development

If you have an instance of the aztec-sandbox running, then you can simply run `yarn test uniswap` to execute the test.

To build and execute the test:

`export FORK_BLOCK_NUMBER=17514288`
`export FORK_URL='https://mainnet.infura.io/v3/9928b52099854248b3a096be07a6b23c'`
`docker build --build-arg COMMIT_TAG=<version of deployed artifacts on npm> .`
`cd ./scripts`
`docker-compose up`
