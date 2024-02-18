---
title: Nodes and Actors
---

To analyze the suitability of different node types for various actor types in the system, let's break down the node types and actor types, considering the characteristics, preferences, abilities, and potential motivations of each actor. 
This will help determine which actors are likely to run which nodes and how they might interact with the Private eXecution Environment (PXE).

## Background

Before diving into Aztec specific data, we take a look at general blockchain nodes. 
It should help us have a shared base, which will later be useful for comparing the Aztec setups, where multiple nodes are involved due to its construction.

### Node Types

The below node types are ordered in resource requirements, and exists for most blockchains. 
All the nodes participate in the peer-to-peer (p2p) network but with varying capacity.

1. **Light Node**:
   - Download and validate headers from the p2p network.
      - Sometimes an "ultra-light" node is mentioned, this is a node that don't validate the headers it receive but just accept it. These typically are connected to a third party trusted by the user to provide valid headers.
   - Stores only the headers.
   - Querying any state not in the header is done by requesting the data from a third party, e.g. Infura or other nodes in the p2p network. Responses are validated with the headers as a trust anchor.
   - Storage requirements typically measured in MBs (< 1GB).
   - Synchronization time is typically measured in minutes.
2. **Full Node**:
   - Receive and validate blocks (header and body) from the p2p network.
   - Stores the complete active state of the chain
   - Typically stores recent state history (last couple of hours is common)
   - Typically stores all blocks since genesis (some pruning might be done)
   - Can respond to queries of current and recent state
   - Storage requirements typically measured in GBs (< 1TB)
   - Synchronization time is typically measured in hours/days.
3. **Archive Node**:
   - Receive and validate blocks (header and body) from the p2p network
   - Stores the full state history of the chain
   - Stores all blocks since genesis
   - Can respond to queries of state at any point in time
   - Storage requirements typically measured in TBs
   - Synchronization time is typically measured in hours/days.

Beyond these node types, there are also nodes that participate in the block production who rely on full or archive nodes to extend the chain. 
In Ethereum these are called validators, but really, any of the nodes above do some validation of the blocks or headers. 
The Ethereum "validator" is really just their block producer.

Block production can generally be split into two parts: 1) building the block and 2) proposing the block.

:::info Proposer-Builder-Separation (PBS)
When these two parts are split between different parties it is often referred to as Proposer-Builder-Separation.
:::

A proposer generally have to put up some value to be able to propose a block and earn rewards from the blocks proposed. 
In PoW this value is burnt $ in electricity, and in PoS it is staked $ which can be "slashed" according to the rules of the chain. 

In the Ethereum world you can say the "validator" is a proposer, that either builds his own blocks or outsource it to someone else, such as [Flashbots](https://www.flashbots.net/).

:::info Blobs
Blobs in Ethereum is a special kind of data that is proven to be available at time of publication and for a short period after. 
After this period (~18 days), the blob is considered shared and can be pruned from the chain. 
It is not needed to synchronize a new node.

Blobs will likely be stored by certain "super-archive" nodes, but not by all archive nodes. 
Meaning that the set of blob-serving nodes likely will be small. 
As the blob-hash is part of the block header, it is easy to validate that some chunk of data was indeed the published blob. 
Relies on an 1/n honesty assumption for access to the blob.
:::

### Actor Types

1. **Mainstream Users**:
   - Currently don't care about technicalities, just want to use the system.
   - Will likely not run any type of node, unless it is bundled with their wallet so they don't even know it is there.
   - Generally don't care about trusting Infura or other third parties blindly.
2. **Enthusiast**: 
   - More knowledgeable and interested in technology than the mainstream user.
   - Willing to invest time and resources but not at a professional level.
   - Likely to run a light node for the extra security.
3. **Power Users**: 
   - Technically proficient and deeply engaged.
   - Likely to have the resources and motivation to run more demanding nodes, and potentially participate in block production.
4. **Developers**: 
   - Highly knowledgeable with technical skills.
   - Interested in detailed state and historical data for development and testing.
5. **Idealists**: 
   - Want maximal autonomy, and are willing to invest resources to achieve it.
   - Will rely entirely on their own nodes and resources for serving their queries, current or historical by running an Archive Node.
   - Likely to run nodes that contribute directly to the blockchain's operation as part of block production.
   - Possibly willing to store all blobs.
6. **Institutions**: 
   - Have significant resources (and potentially technical) capabilities.
   - Interested in robust participation, and potentially participate in block production for financial gain.

## Aztec Relevant

In the following section, we are following the assumption that Aztec is running on top of Ethereum and using some DA to publish its data.

Beyond the [state](./../state/index.md) of the chain an Aztec node also maintains a database of encrypted data and their tags - an index of tags. 
The index is used when a user wishes to retrieve notes that belong to them. 
The index is built by the node as it receives new blocks from the network.

If the node have the full index it can serve any user that wants to retrieve their notes. This will be elaborated further in [responding to queries](#responding-to-queries).
A node could be configured to only build and serve the index for a subset of the tags and notes. 
For example, a personal node could be configured to only build the index for the notes that belong to the owner based on their [tag derivation](./../private-message-delivery/private-msg-delivery.md#note-tagging).

If the node is intended only for block production, it can skip the index entirely.

### Synchronizing An Aztec Node

To synchronize an Aztec full- or archive-node, we need the blocks to build state and index. We have two main options for doing so:
- Aztec nodes which is running dependency-minimized (i.e. not relying on a third party for data), and agree with the bridge on what is canonical, can retrieve the headers directly from their Ethereum node and the matching bodies from their node on the DA network. If blobs are used for DA, both of these could be the same node. The node will build the state and index as it receives the blocks.
- Aztec nodes that are not running dependency-minimized can rely on third parties for retrieving state and block bodies and use Ethereum as a "trust-anchor". Here meaning that the Aztec node could retrieve the full state from IPFS and then validate it against the headers it receives from the Ethereum node - allowing a quick synchronization process. Since the index is not directly part of state, its validity cannot be validated as simple as the state. The index will have to be built from the blocks (validating individual decrypted messages can be done against the state).

An Aztec light-node is a bit different, because it does not store the actual state *nor* index, only the headers.
This means that it will follow the same model as Ethereum light-nodes, where it can retrieve data by asking third parties for it and then validating that it is in the state (anchored by the headers stored).

:::info Aztec-specific Ultra Light nodes
For users following the validating bridge as the canonical rollup, the bridge IS the light node, so they can get away with not storing anything, and just download the header data from the bridge. 
Since the [archive](./../state/archive.md) is stored in the bridge, an Aztec ultra-light node can validate membership proofs against it without needing the exact header (given sufficient membership proofs are provided). 
This essentially enable instant sync for ultra-light nodes -> if you either run an Ethereum node or trust a third party.
:::

### Responding to Queries

In the following, we will briefly outline how the different type of Aztec node will respond to different kinds of queries. Namely we will consider the following queries:
- Requesting current public state and membership proofs
- Requesting historical public state and membership proofs
- Requesting current membership proof of note hash, nullifier, l1 to l2 message or contract 
- Requesting historical membership proof of note hash, nullifier, l1 to l2 message or contract 
- Requesting encrypted notes for a given tag

#### Light-Node

As mentioned, light-nodes will be retrieving data from a third party, and then validating it against the headers at its disposal.
Note that if we have the latest archive hash, two extra inclusion proofs can be provided to validate the response against the archive as well without needing any other data.

If we don't care about other people seeing what data we are retrieving, we can simply ask the third parties directly. 
However, when dealing with private information (e.g. notes and their inclusion proofs), you need to be careful about how you retrieve it - you need to use some form of private information retrieval (PIR) service. 

The exact nature of the service can vary and have not been decided yet, but it could be some form of Oblivious Message Retrieval (OMR) service. 
This is a service that allows you to retrieve a message from a database without revealing which message you are retrieving.
An OMR service could be run by a third party who runs a full or archive node and is essentially just a proxy for other nodes to fetch information from the database.
There might exists multiple OMR services, some with access to historical state and some without.

Assuming that OMR is used, the user would occasionally use the light-node to request encrypted notes from the OMR service.
These notes can be decrypted by the user, and used to compute the note hash, for which they will need an inclusion proof to spend the note.
When the user wish to spend the note, they can request the inclusion proof from the OMR service, which they can then validate against the headers they have stored.

:::info Note Discovery Issue
The issue of how to discover which notes belong to you is not solved by the Aztec protocol currently, and we have a [Request For Proposals](https://forum.aztec.network/t/request-for-proposals-note-discovery-protocol/2584) on the matter.
:::

#### Full-Node
Can satisfy any query on current data, but will have to retrieve historical membership proofs from a third party or recompute the requested state based on snapshots and blocks.

Can utilize the same protocol as the Light-node for retrieving data that it doesn't already have.

#### Archive-Node
With a fully synched archive node you can respond to any query using your own data - no third party reliance.

### Private eXecution Environment (PXE)

The Aztec PXE is required to do anything meaningful for an end-user. 
It is the interface between the user and the Aztec network and acts as a private enclave that runs on the end-user's machine.

While it is responsible for executing contracts and building proofs for the user's transactions privately, it needs data from the node to do so. 
It can be connected to any of the above types of nodes, inheriting their assumptions. 
From the point of view of the PXE, it is connected to a node, and it does not really care which one.

When requesting encrypted notes from the node the PXE will decrypt it and store the decrypted notes in its own database. 
This database is used to build proofs for the user's transactions. 

From most end users points of view, the PXE will be very close to a wallet.

### Bundling the PXE with an Aztec ultra-light node

A deployment of the PXE could be bundled together with an aztec ultra-light node which in turn is connected to "some" Ethereum node. 
The ethereum node could simply be infura, and then the ultra-light node handles its connections to the OMR service separately. 

This setup can be made very light-weight and easy to use for the end-user, who might not be aware that they are running a node at all.

The reasoning that the ultra-light node should be bundled with an internal ultra-light node by default instead of simply using an RPC endpoint is to avoid users "missing" the OMR services and leak private information.

If the user is running a node themselves, on a separate device or whatever, they could connect the PXE to that node instead of the bundled one.