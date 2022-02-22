# Questions

A list of random thoughts and questions that haven't been fully considered yet.

This isn't a comprehensive list of questions - there are loads dotted around these pages.

- Where to develop the code?
  - Will need to make significant edits to Falafel.
  - Aztec Connect circuits aren't needed.
- How should I proceeed?
  - I could plan out lots of distinct tasks.
  - Roadmap:
    - Releases with milestones of functionality?
    - One final big release at the end?
    - E.g. Charlie suggested ignoring Public circuit functionality altogether initially.
  - I can start writing kernel circuits, rollup circuits, making changes to falafel, writing smart contracts.
- Components:
  - Smart contract language: Noir?
  - Testing framework: also Noir?
  - Rollup provider full node: Falafel?
    - Receives tx requests
    - Creates rollups
      - Coordinates delegation of proofs (future enhancement)
    - Tracks the entire L2 state
  - Rollup delegate node: Falafel in  different 'mode'.
    - Can be given / takes instructions to generate proofs for the rollup.
    - Incentivisation mechanism needed? I think we can avoid this.
  - User node: Also Falafel in a different 'mode', or separate?
    - For generating client-side proofs
    - Submitting proofs to the Rollup Provider pool
    - For keeping minimal user state witnesses and querying stuff.
  - User node: non-browser version?
  - web3.js/ethers.js pacakge to convert JS tx requests into the Aztec 3 ABI format.
- Noir - how 'high-level' should it be?
  - One extreme: it 'abstracts away' all of Aztec 3's architecture from the developer.
    - Problem: one API for developer, and another ABI for the underlying circuit.
      - Either need Noir plugged into a DB (but I'd prefer it to be stateless)
      - Or need an ethers.js program to take a JS tx, query the falafel DB, then send an ABI-style tx to Noir.
    - If we make architecture changes to the Public Inputs ABIs (which we will when optimising), Noir would need corresponding changes.
  - Other extreme: a developer needs to write, in detail, circuits which conform to the Aztec 3 spec.
- Once the HONK backend is ready, will we be able to simply 'swap' our kernel circuits' backend from PLONK to HONK?
- Ask about callbacks:
  - One-time-use callbacks (my preference) vs callback behaviour designed by each dev.
  - Dedicated, decorated callback functions vs general functions that don't even _know_ they're a callback.
    - If decorated as callback, the function can contain L1 result reconciliation logic.
    - If general functions, then the L1 result reconciliation must happen in the kernel circuit / base rollup circuit.
- Meh: Take a look at an [alternative suggestion for callbacks](./architecture/contracts/l1-calls.md#alternative-approach-to-callbacks), and decide which is best.
- Decentralisation Roadmap:
  - Rollup provider selection process?
    - Aztec Token tokenomics.
  - Tx pool - p2p messaging layer? Or direct messaging (if the Rollup Provider is the same for a period of time)?
- Revisit discussion on fees and attributing L2 fees payments to provers based on their proving work.
- Use ETH as L2 gas payment currency, or allow any general function to be labelled as a 'fee paying' currency?
  - If we have specific gas costs relating to SLOAD / GEN_PROOF, then these will need to be paid in ETH surely?
- Ask about dynamic arrays and what the limitations are in UltraPlonk.
  - Pushing to all of these callStacks is going to expensive (constraint-wise), especially if we want to tightly pack pushes. Lots of O(n^2) searching for correct positions in arrays.
  - Unless we can do some clever dynamic array stuff with ultraplonk / honk?
- Do we need to emit all events to a special log that can be parsed? Like an L2 version of Ethereum's logs?
  - E.g. how can app-specific javascript code 'react' to an event emitted by an L2 function?
  - How does Ethereum do it?
