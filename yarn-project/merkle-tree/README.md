# Merkle Tree

## Overview

The Merkle Trees package contains typescript implementations of the various Merkle Trees required by the system.

- Append only 'Standard' merkle trees. New values are inserted into the next available leaf index. Values are never updated.
- Indexed trees are also append only in nature but retain the ability to update leaves. The reason for this is that the Indexed Tree leaves not only store the value but the index of the next highest leaf. New insertions can require prior leaves to be updated.
- Sparse trees that can be updated at any index. The 'size' of the tree is defined by the number of non-empty leaves, not by the highest populated leaf index as is the case with a Standard Tree.

## Implementations

All implementations require the following arguments:

- An instance of level db for storing information at a tree node level as well as some tree level metadata.
- An instance of a Hasher object. Used for hashing the nodes of the Merkle Tree.
- A name, for namespacing the tree's nodes within the db.
- The depth of the tree, this is exclusive of the root.

Implementations have commit/rollback semantics with modifications stored only in memory until being committed. Rolling back returns the tree discards the cached modifications and returns the tree to it's previous committed state.

## Tree Operations

The primary operations available on the various implementations are:

- Create a new, empty tree
- Restore a tree from the provided DB
- Append new leaves (Standard and Indexed trees only)
- Update a leaf (Sparse tree and Indexed trees only)
- Retrieve the number of leaves. This is the number of non empty leaves for a Sparse tree and the highest populated index for Standard and Indexed trees.
- Commit modifications
- Rollback modifications
- Retrieve a Sibling Path for a given index. For performing Merkle proofs it is necessary to derive the set of nodes from a leaf index to the root of the tree, known as the 'hash path'. Given a leaf, it is therefore required to have the 'sibling' node at each tree level in order for the hash path to be computed.

Note: Tree operations are not 'thread' safe. Operations should be queued or otherwise executed serially to ensure consistency of the data structures and any data retrieved from them.

## Building/Testing

To build the package, execute `yarn build` in the root.

To run the tests, execute `yarn test`.
