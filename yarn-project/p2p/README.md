# P2P

This package defined an interface for a client that keeps track of Aztec transaction requests.

When a transaction goes through (mined in a block), the p2p client is also responsible for reconciling the pool. i.e.

- Check for new confirmed blocks
- When new block is mined, go through its transactions and remove them from the pool.

A very naive, in-memory implementation of the P2P client is also included in the package.
It's currently used by the implementations of the sequencer and node.
