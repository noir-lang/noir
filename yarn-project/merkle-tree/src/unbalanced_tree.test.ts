import { sha256Trunc } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type FromBuffer } from '@aztec/foundation/serialize';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type Hasher } from '@aztec/types/interfaces';

import { SHA256Trunc } from './sha_256.js';
import { StandardTree } from './standard_tree/standard_tree.js';
import { UnbalancedTree } from './unbalanced_tree.js';

const noopDeserializer: FromBuffer<Buffer> = {
  fromBuffer: (buffer: Buffer) => buffer,
};

// Follows sol implementation and tests in UnbalancedMerkle.t.sol
describe('Wonky tree', () => {
  let hasher: Hasher;
  let tree: UnbalancedTree<Buffer>;
  let leaves: Buffer[];

  const createAndFillTree = async (size: number) => {
    const depth = Math.ceil(Math.log2(size));
    const tree = new UnbalancedTree(hasher, `test`, depth, noopDeserializer);
    const leaves = Array(size)
      .fill(0)
      .map((_, i) => sha256Trunc(new Fr(i).toBuffer()));
    // For the final test, we make the final (shifted up) leaf be H(1, 2), so we can calculate the root
    // with a standard tree easily.
    if (leaves[30]) {
      leaves[30] = hasher.hash(new Fr(1).toBuffer(), new Fr(2).toBuffer());
    }
    await tree.appendLeaves(leaves);
    return { tree, leaves };
  };

  beforeAll(() => {
    hasher = new SHA256Trunc();
  });

  // Example - 2 txs:
  //
  //   root
  //  /     \
  // base  base
  describe('2 Transactions', () => {
    beforeAll(async () => {
      const res = await createAndFillTree(2);
      tree = res.tree;
      leaves = res.leaves;
    });

    it("Shouldn't accept more leaves", () => {
      expect(() => tree.appendLeaves([Buffer.alloc(32)])).toThrow(
        "Can't re-append to an unbalanced tree. Current has 2 leaves.",
      );
    });

    it('Correctly computes tree information', () => {
      expect(tree.getNumLeaves()).toEqual(BigInt(leaves.length));
      expect(tree.getDepth()).toEqual(1);
      expect(tree.findLeafIndex(leaves[0])).toEqual(0n);
    });

    it('Correctly computes root', () => {
      const root = tree.getRoot();
      const expectedRoot = sha256Trunc(Buffer.concat([leaves[0], leaves[1]]));
      expect(root).toEqual(expectedRoot);
    });

    it('Correctly computes sibling path', async () => {
      const sibPath = await tree.getSiblingPath(BigInt('0x' + leaves[0].toString('hex')));
      expect(sibPath.pathSize).toEqual(1);
      const expectedSibPath = [leaves[1]];
      expect(sibPath.toBufferArray()).toEqual(expectedSibPath);
    });
  });

  // Example - 3 txs:
  //
  //        root
  //     /        \
  //   merge     base
  //  /     \
  // base  base
  describe('3 Transactions', () => {
    beforeAll(async () => {
      const res = await createAndFillTree(3);
      tree = res.tree;
      leaves = res.leaves;
    });

    it('Correctly computes tree information', () => {
      expect(tree.getNumLeaves()).toEqual(BigInt(leaves.length));
      expect(tree.getDepth()).toEqual(2);
      expect(tree.findLeafIndex(leaves[0])).toEqual(0n);
    });

    it('Correctly computes root', () => {
      const root = tree.getRoot();
      const mergeNode = sha256Trunc(Buffer.concat([leaves[0], leaves[1]]));
      const expectedRoot = sha256Trunc(Buffer.concat([mergeNode, leaves[2]]));
      expect(root).toEqual(expectedRoot);
    });

    it('Correctly computes sibling path', async () => {
      const sibPath = await tree.getSiblingPath(BigInt('0x' + leaves[0].toString('hex')));
      expect(sibPath.pathSize).toEqual(2);
      const expectedSibPath = [leaves[1], leaves[2]];
      expect(sibPath.toBufferArray()).toEqual(expectedSibPath);
    });
  });

  // Example - 5 txs:
  //
  //                  root
  //             /            \
  //          merge           base
  //      /          \
  //   merge        merge
  //  /     \      /     \
  // base  base  base   base
  describe('5 Transactions', () => {
    beforeAll(async () => {
      const res = await createAndFillTree(5);
      tree = res.tree;
      leaves = res.leaves;
    });

    it('Correctly computes tree information', () => {
      expect(tree.getNumLeaves()).toEqual(BigInt(leaves.length));
      expect(tree.getDepth()).toEqual(3);
      expect(tree.findLeafIndex(leaves[0])).toEqual(0n);
    });

    it('Correctly computes root', () => {
      const root = tree.getRoot();
      let leftMergeNode = sha256Trunc(Buffer.concat([leaves[0], leaves[1]]));
      const rightMergeNode = sha256Trunc(Buffer.concat([leaves[2], leaves[3]]));
      leftMergeNode = sha256Trunc(Buffer.concat([leftMergeNode, rightMergeNode]));
      const expectedRoot = sha256Trunc(Buffer.concat([leftMergeNode, leaves[4]]));
      expect(root).toEqual(expectedRoot);
    });

    it('Correctly computes sibling path', async () => {
      const sibPath = await tree.getSiblingPath(BigInt('0x' + leaves[0].toString('hex')));
      expect(sibPath.pathSize).toEqual(3);
      const expectedSibPath = [leaves[1], sha256Trunc(Buffer.concat([leaves[2], leaves[3]])), leaves[4]];
      expect(sibPath.toBufferArray()).toEqual(expectedSibPath);
    });
  });

  // Example - 6 txs:
  //
  //                  root
  //             /            \
  //         merge4           merge3
  //      /          \        /    \
  //   merge1       merge2  base  base
  //  /     \      /     \
  // base  base  base   base
  describe('6 Transactions', () => {
    beforeAll(async () => {
      const res = await createAndFillTree(6);
      tree = res.tree;
      leaves = res.leaves;
    });

    it('Correctly computes tree information', () => {
      expect(tree.getNumLeaves()).toEqual(BigInt(leaves.length));
      expect(tree.getDepth()).toEqual(3);
      expect(tree.findLeafIndex(leaves[0])).toEqual(0n);
    });

    it('Correctly computes root', () => {
      const root = tree.getRoot();
      let leftMergeNode = sha256Trunc(Buffer.concat([leaves[0], leaves[1]]));
      let rightMergeNode = sha256Trunc(Buffer.concat([leaves[2], leaves[3]]));
      leftMergeNode = sha256Trunc(Buffer.concat([leftMergeNode, rightMergeNode]));
      rightMergeNode = sha256Trunc(Buffer.concat([leaves[4], leaves[5]]));
      const expectedRoot = sha256Trunc(Buffer.concat([leftMergeNode, rightMergeNode]));
      expect(root).toEqual(expectedRoot);
    });

    it('Correctly computes sibling path', async () => {
      const sibPath = await tree.getSiblingPath(BigInt('0x' + leaves[0].toString('hex')));
      expect(sibPath.pathSize).toEqual(3);
      const expectedSibPath = [
        leaves[1],
        sha256Trunc(Buffer.concat([leaves[2], leaves[3]])),
        sha256Trunc(Buffer.concat([leaves[4], leaves[5]])),
      ];
      expect(sibPath.toBufferArray()).toEqual(expectedSibPath);
    });
  });

  // Example - 7 txs:
  //
  //                     root
  //             /                  \
  //         merge3                merge5
  //      /          \             /    \
  //   merge1       merge2      merge4  base
  //  /     \      /     \      /    \
  // base  base  base   base  base  base
  describe('7 Transactions', () => {
    let secondMergeNode: Buffer;
    let fifthMergeNode: Buffer;
    beforeAll(async () => {
      const res = await createAndFillTree(7);
      tree = res.tree;
      leaves = res.leaves;
    });

    it('Correctly computes tree information', () => {
      expect(tree.getNumLeaves()).toEqual(BigInt(leaves.length));
      expect(tree.getDepth()).toEqual(3);
      expect(tree.findLeafIndex(leaves[0])).toEqual(0n);
    });

    it('Correctly computes root', () => {
      const root = tree.getRoot();
      const firstMergeNode = sha256Trunc(Buffer.concat([leaves[0], leaves[1]]));
      secondMergeNode = sha256Trunc(Buffer.concat([leaves[2], leaves[3]]));
      const thirdMergeNode = sha256Trunc(Buffer.concat([firstMergeNode, secondMergeNode]));
      const fourthMergeNode = sha256Trunc(Buffer.concat([leaves[4], leaves[5]]));
      fifthMergeNode = sha256Trunc(Buffer.concat([fourthMergeNode, leaves[6]]));
      const expectedRoot = sha256Trunc(Buffer.concat([thirdMergeNode, fifthMergeNode]));
      expect(root).toEqual(expectedRoot);
    });

    it('Correctly computes sibling path', async () => {
      const sibPath = await tree.getSiblingPath(BigInt('0x' + leaves[0].toString('hex')));
      expect(sibPath.pathSize).toEqual(3);
      const expectedSibPath = [leaves[1], secondMergeNode, fifthMergeNode];
      expect(sibPath.toBufferArray()).toEqual(expectedSibPath);
    });
  });

  // Example - 31 txs:
  // The same as a standard 32 leaf balanced tree, but with the last 'leaf' shifted up one.
  describe('31 Transactions', () => {
    let stdTree: StandardTree;
    beforeAll(async () => {
      const res = await createAndFillTree(31);
      tree = res.tree;
      leaves = res.leaves;
      stdTree = new StandardTree(openTmpStore(true), hasher, `temp`, 5, 0n, noopDeserializer);
      // We have set the last leaf to be H(1, 2), so we can fill a 32 size tree with:
      await stdTree.appendLeaves([...res.leaves.slice(0, 30), new Fr(1).toBuffer(), new Fr(2).toBuffer()]);
    });

    it('Correctly computes tree information', () => {
      expect(tree.getNumLeaves()).toEqual(BigInt(leaves.length));
      expect(tree.getDepth()).toEqual(5);
      expect(tree.findLeafIndex(leaves[0])).toEqual(0n);
    });

    it('Correctly computes root', () => {
      const root = tree.getRoot();
      const expectedRoot = stdTree.getRoot(true);
      expect(root).toEqual(expectedRoot);
    });

    it('Correctly computes sibling paths', async () => {
      let sibPath = await tree.getSiblingPath(BigInt('0x' + leaves[0].toString('hex')));
      let expectedSibPath = await stdTree.getSiblingPath(0n, true);
      expect(sibPath).toEqual(expectedSibPath);
      sibPath = await tree.getSiblingPath(BigInt('0x' + leaves[27].toString('hex')));
      expectedSibPath = await stdTree.getSiblingPath(27n, true);
      expect(sibPath).toEqual(expectedSibPath);
    });
  });
});
