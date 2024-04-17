# A minimal example of nested recursion

## About

The code in this project shows recursive verification of Noir functions.

The primary function is simply addition, and verifies the re-calculation of one path up a binary tree from two leaf nodes.
A more useful application of this design would be proving the hash of data in a merkle tree (leaf nodes), up to the merkle root. Amortizing the cost of proving each hash per nested call.

## The circuits
The function doing the calculation, in this case addition, is in the sumLib. It is used in both recursive circuits: `sum` and `recursiveLeaf`.

## Verification
Results of a call to `sum` are verified in `recursiveLeaf`, which itself also calls `sum` again. The results of the `recursiveLeaf` call are then verified in `recursiveNode`.

That is:
- `recursiveNode` verifies `recursiveLeaf` artifacts
- `recursiveLeaf` verifies `sum` artifacts

## Using this project
- Install dependencies: `yarn`
- Compile all the things: `yarn compile:all`
- Run main typescript: `yarn start`
