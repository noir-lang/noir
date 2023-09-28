# Aztec Node

The Aztec Node implements a sequencer node in the network, and is currently meant to be used for local development and testing. The Node is the entrypoint for creating and starting a new Sequencer client with default components (a local P2P client, an in-memory merkle tree database, etc). 

The Node also exposes methods that are consumed by the client (see `pxe`), such as querying network info or submitting a transaction. As Aztec evolves beyond local development, these methods will be accessible via a JSON-RPC API or similar. Refer to the `end-to-end` tests for examples on how to initialize an Aztec Node and use it along with a Private eXecution Environment (PXE).

## Development

Start by running `bootstrap.sh` in the project root.

To build the package, run `yarn build` in the root.

To watch for changes, `yarn build:dev`.

To run the tests, execute `yarn test`.