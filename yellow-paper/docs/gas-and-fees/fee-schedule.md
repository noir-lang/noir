# Fee Schedule

:::info
This section is a placeholder, we will flesh this out in much greater detail when we come to profile operations and assign gas costs
:::

<!-- prettier-ignore -->
| Action | Resource Domain | Consumption Calculation | Comment |
| -------- | -------- | -------- | ------- |
| Verifying the private kernel proof | L2 | Fixed L2/Transaction | |
| Verifying each nullifier against the world state    | L2     | Fixed L2/Tx nullifier     | |
| Verifying each nullifier against others in the same block     | L2     | Fixed L2/Tx nullifier     | Whilst not strictly a fixed cost, this would need to be allocated as a fixed cost as it depends on the composition of the rollup |
| Verifying log preimages against the sha256 log hashes contained in the private kernel public inputs | L2 | L2 gas per pre-image field | |
| Verifying contract deployment data against the sha256 hash of this data contained in the private kernel public inputs | L2 | L2 gas per hash | |  
| Publishing contract data to DA     | DA     | DA gas per byte     | |
| Publishing state updates to DA     | DA     | DA gas per byte     | |
| Publishing notes/tags to DA    | DA     | DA gas per byte     | |
| Publishing L2->L1 messages | L1 | Calldata gas per byte + processing & storing of the message | |
| Public function execution    | L2     | L2 gas per function opcode     | |
| Proving the public VM circuit for a public function     | L2     | Fixed L2/Tx public function     | |
| Proving the public kernel circuit for a public function    | L2     | Fixed L2/Tx public function   | |
| Proving the base rollup circuit | L2 | Fixed L2/Transaction |
| Proving the merge rollup circuit | L2 | Amortized L2/Transaction |
| Proving the root rollup circuit | L2 | Amortized L2/Transaction |
| Publishing the block header to L1 | L1 | Amortized L1/Transaction |
| Verifying the rollup proof | L1 | Amortized L1/Transaction |