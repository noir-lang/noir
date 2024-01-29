---
title: Cryptography Roadmap
---

The cryptography team is currently working on [Barretenberg here.](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg)

## R&D projects

Publish the Honk paper, describing practical considerations for constructing our cutting-edge proving system Honk along with formal proofs of its security properties.

## Honk

Honk is a sumcheck-based zk-SNARK protocol with blazing-fast zk proof construction. We need Honk to allow users to prove correct execution of complicated, multi-step computations using recursion in a resource constraint environment like a cell phone. This is necessary for our mission, because we need to make sure our users' sensitive information never leaves their devices!

List of Honk projects:

- Completed: basic Honk prover and verifier with respectable construction and verification speeds, but no optimization.
- Upcoming:
  - Bringing "Ultra" functionality to Honk: lookup tables, efficient range constraints, RAM, ROM, and more will result in orders-of-magnitude improvements to Honk's prover times.
  - Recursion using cycles of curves will allow for efficient recursive verification of Honk proofs. Using this technique will lower the barrier to entry of our rollup providers, resulting in a more robust set of providers and greater security for the Aztec network.

## Goblin projects

Goblin is a deferred verification framework that allows for an order-of-magnitude increase in the complexity of computations that Aztec users can execute with full privacy. This corresponds to a 10x increase in the expressivity of Noir programs that can be run in practice without melting anybody's favorite phone or laptop. Read more [here](https://hackmd.io/@aztec-network/B19AA8812).
