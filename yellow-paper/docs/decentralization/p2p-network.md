# P2P Network

## Requirements for a P2P Network

When a rollup is successfully published, the state transitions it produces are published along with it, making them publicly available. This broadcasted state does not depend on the Aztec network for its persistence or distribution. Transient data however, such as pending user transactions for inclusion in future rollups, does rely on the network for this. It is important that the network provides a performant, permissionless and censorship resistant mechanism for the effective propagation of these transactions to all network participants. Without this, transactions may be disadvantaged and the throughput of the network will deteriorate.

Other data that may be transmitted over the network are the final rollup proofs to be submitted to the rollup contract, however the size and rate of these payloads should not make any meaningful impact on the bandwidth requirements.

### Network Participants

For the purpose of this discussion, we define the 'Aztec Network' as the set of components required to ensure the continual distribution of user transactions and production of rollups. The participants in such a network are:

- Sequencers - responsible for selecting transactions from the global pool and including them in rollups
- Provers - responsible for generating zk-proofs for the transaction and rollup circuits
- Transaction Pool Nodes - responsible for maintaining a local representation of the pending transaction pool
- Bootnodes - responsible for providing an entrypoint into the network for new participants

Sequencers and Provers will likely run their own transaction pools but it is important that the ability to do so is not limited to these participants. Anyone can operate a transaction pool, providing increased privacy and censorship resistance.

Client PXEs will not interact directly with the network but instead via instances of the Aztec Node and it's JSON RPC interface. The Aztec Node in turn will publish user transactions to the network.

### Network Topology and Transaction Submission

The network will likely be based on the LibP2P specification.

#### Discovery

When a node such as a sequencer joins the network for the first time it will need to contact a bootnode. This will be one of a hardcoded set of nodes whose sole purpose is to provide an initial set of peers to connect with. Once a node has made contact with and stored address data for a number of peers it should no longer need to contact the bootnodes, provided it's peers remain available.

#### Gossip

Transactions will need to be propagated throughout the network, to every participant. Due to the size of the transaction payloads it will be necessary to ensure that transaction propagation is performed as efficiently as possible taking steps to reduce the amount of redundant data transfer.

#### Client Interactions

Aztec Node instances will offer a JSON RPC interface for consumption by a user's PXE. Part of this API will facilitate transaction submission directly to the node which will then forward it to the network via the transaction pool.

![P2P Network](./images/network.png)

### Network Bandwidth

Transactions are composed of several data elements and can vary in size. The transaction size is determined largely by the private kernel proof and whether the transaction deloys any public bytecode. A typical transaction that emits a private note and an unencrypted log, makes a public call and contains a valid proof would consume approximately 40Kb of data. A transaction that additionally deploys a contract would need to transmit the public bytecode on top of this.

| Element                                      | Size  |
| -------------------------------------------- | ----- |
| Public Inputs, Public Calls and Emitted Logs | ~8Kb  |
| Private Kernel Proof                         | ~32Kb |

If we take 2 values of transaction throughput of 10 and 100 transactions per second, we can arrive at average network bandwidth requirements of 400Kb and 4000Kb per second respectively.

### Sequencer to Prover Communication

Proving is an out-of-protocol activity. The nature of the communication between sequencers and provers will depend entirely on the prover/s selected by the sequencer. Provers may choose to run their own Transaction Pool Node infrastructure so that they are prepared for generating proofs and don't need to receive this data out-of-band.

Although proving is an out-of-protocol activity, it may be necessary for the final rollup proof to be gossiped over the P2P network such that anyone can submit it to the rollup contract.
