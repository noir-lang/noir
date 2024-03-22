/** A simple immutable Merkle tree container. Use a MerkleTreeCalculator to create a new instance from a set of leaves. */
export class MerkleTree {
  constructor(public readonly height: number, public readonly nodes: Buffer[]) {
    const expectedNodeCount = 2 ** (height + 1) - 1;
    if (nodes.length !== expectedNodeCount) {
      throw new Error(`Invalid node count for Merkle tree: got ${nodes.length} but expected ${expectedNodeCount}`);
    }
  }

  get root(): Buffer {
    return this.nodes[this.nodes.length - 1];
  }

  get leaves(): Buffer[] {
    return this.nodes.slice(0, 2 ** this.height);
  }

  /** Returns a sibling path to the given element or to the element in the given index. */
  public getSiblingPath(leafIndex: number): Buffer[];
  public getSiblingPath(leaf: Buffer): Buffer[];
  public getSiblingPath(leafIndexOrLeaf: number | Buffer): Buffer[] {
    if (Buffer.isBuffer(leafIndexOrLeaf)) {
      return this.getSiblingPath(this.getIndex(leafIndexOrLeaf));
    }
    const leafIndex = leafIndexOrLeaf;
    if (leafIndex < 0 || leafIndex >= 2 ** this.height) {
      throw new Error(`Invalid leaf index: got ${leafIndex} but leaves count is ${2 ** this.height}`);
    }
    const tree = this.nodes;
    let rowSize = Math.ceil(tree.length / 2);
    let rowOffset = 0;
    let index = leafIndex;
    const siblingPath: Buffer[] = [];
    while (rowSize > 1) {
      const isRight = index & 1;
      siblingPath.push(tree[rowOffset + index + (isRight ? -1 : 1)]);
      rowOffset += rowSize;
      rowSize >>= 1;
      index >>= 1;
    }
    return siblingPath;
  }

  /** Returns the leaf index for a given element. */
  public getIndex(element: Buffer) {
    return this.leaves.findIndex(leaf => leaf.equals(element));
  }

  /** Returns a nice string representation of the tree, useful for debugging purposes. */
  public drawTree() {
    const levels: string[][] = [];
    const tree = this.nodes;
    const maxRowSize = Math.ceil(tree.length / 2);
    let paddingSize = 1;
    let rowSize = maxRowSize;
    let rowOffset = 0;
    while (rowSize > 0) {
      levels.push(
        tree
          .slice(rowOffset, rowOffset + rowSize)
          .map(n => n.toString('hex').slice(0, 8) + ' '.repeat((paddingSize - 1) * 9)),
      );
      rowOffset += rowSize;
      paddingSize <<= 1;
      rowSize >>= 1;
    }
    return levels
      .reverse()
      .map(row => row.join(' '))
      .join('\n');
  }
}
