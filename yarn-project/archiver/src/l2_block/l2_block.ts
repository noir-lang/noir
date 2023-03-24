import { numToUInt32BE } from '@aztec/foundation';

/**
 * A snapshot of an append only tree.
 */
export type AppendOnlyTreeSnapshot = {
  /**
   * The root of the tree, as a field element.
   */
  root: Buffer;
  /**
   * The next available index in the tree.
   */
  nextAvailableLeafIndex: number;
};

/**
 * A contract data blob, containing L1 and L2 addresses.
 */
export type ContractData = {
  /**
   * The L2 address of the contract, as a field element (32 bytes).
   */
  aztecAddress: Buffer;
  /**
   * The L1 address of the contract, (20 bytes).
   */
  ethAddress: Buffer;
};

/**
 * The data that makes up the rollup proof, with encoder decoder functions.
 */
export class L2Block {
  /**
   * A yeet to go with the block.
   */
  public yeet?: Buffer;

  /**
   * Construct a new L2Block object.
   * The data that goes into the rollup, BUT without the proof.
   * @param number - The number of the L2 block.
   * @param startPrivateDataTreeSnapshot - The tree snapshot of the private data tree at the start of the rollup.
   * @param startNullifierTreeSnapshot - The tree snapshot of the nullifier tree at the start of the rollup.
   * @param startContractTreeSnapshot - The tree snapshot of the contract tree at the start of the rollup.
   * @param startTreeOfHistoricPrivateDataTreeRootsSnapshot - The tree snapshot of the historic private data tree roots at the start of the rollup.
   * @param startTreeOfHistoricContractTreeRootsSnapshot - The tree snapshot of the historic contract tree roots at the start of the rollup.
   * @param endPrivateDataTreeSnapshot - The tree snapshot of the private data tree at the end of the rollup.
   * @param endNullifierTreeSnapshot - The tree snapshot of the nullifier tree at the end of the rollup.
   * @param endContractTreeSnapshot - The tree snapshot of the contract tree at the end of the rollup.
   * @param endTreeOfHistoricPrivateDataTreeRootsSnapshot - The tree snapshot of the historic private data tree roots at the end of the rollup.
   * @param endTreeOfHistoricContractTreeRootsSnapshot - The tree snapshot of the historic contract tree roots at the end of the rollup.
   * @param newCommitments - The commitments to be inserted into the private data tree.
   * @param newNullifiers - The nullifiers to be inserted into the nullifier tree.
   * @param newContracts - The contracts leafs to be inserted into the contract tree.
   * @param newContractData - The aztec_address and eth_address for the deployed contract and its portal contract.
   */
  constructor(
    public number: number,
    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    public startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,
    public endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public newCommitments: Buffer[],
    public newNullifiers: Buffer[],
    public newContracts: Buffer[],
    public newContractData: ContractData[],
  ) {}

  /**
   * Sets the yeet on this block.
   * @param yeet - The yeet to set.
   */
  setYeet(yeet: Buffer) {
    this.yeet = yeet;
  }

  /**
   * Encode the L2 block data into a buffer that can be pushed to the rollup contract.
   * @returns The encoded L2 block data.
   */
  encode(): Buffer {
    return Buffer.concat([
      numToUInt32BE(Number(this.number)),
      appendOnlyTreeSnapshotToBuffer(this.startPrivateDataTreeSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.startNullifierTreeSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.startContractTreeSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.startTreeOfHistoricPrivateDataTreeRootsSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.startTreeOfHistoricContractTreeRootsSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.endPrivateDataTreeSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.endNullifierTreeSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.endContractTreeSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.endTreeOfHistoricPrivateDataTreeRootsSnapshot),
      appendOnlyTreeSnapshotToBuffer(this.endTreeOfHistoricContractTreeRootsSnapshot),
      numToUInt32BE(this.newCommitments.length),
      ...this.newCommitments,
      numToUInt32BE(this.newNullifiers.length),
      ...this.newNullifiers,
      numToUInt32BE(this.newContracts.length),
      ...this.newContracts,
      ...this.newContractData.map(contractData => contractDataToBuffer(contractData)),
    ]);
  }

  /**
   * Decode the L2 block data from a buffer.
   * @param encoded - The encoded L2 block data.
   * @returns The decoded L2 block data.
   */
  static decode(encoded: Buffer) {
    let offset = 0;
    const blockNum = encoded.readUInt32BE(offset);
    offset += 4;
    const startPrivateDataTreeSnapshot = bufferToAppendOnlyTreeSnapshot(encoded.subarray(offset, offset + 36));
    offset += 36;
    const startNullifierTreeSnapshot = bufferToAppendOnlyTreeSnapshot(encoded.subarray(offset, offset + 36));
    offset += 36;
    const startContractTreeSnapshot = bufferToAppendOnlyTreeSnapshot(encoded.subarray(offset, offset + 36));
    offset += 36;
    const startTreeOfHistoricPrivateDataTreeRootsSnapshot = bufferToAppendOnlyTreeSnapshot(
      encoded.subarray(offset, offset + 36),
    );
    offset += 36;
    const startTreeOfHistoricContractTreeRootsSnapshot = bufferToAppendOnlyTreeSnapshot(
      encoded.subarray(offset, offset + 36),
    );
    offset += 36;
    const endPrivateDataTreeSnapshot = bufferToAppendOnlyTreeSnapshot(encoded.subarray(offset, offset + 36));
    offset += 36;
    const endNullifierTreeSnapshot = bufferToAppendOnlyTreeSnapshot(encoded.subarray(offset, offset + 36));
    offset += 36;
    const endContractTreeSnapshot = bufferToAppendOnlyTreeSnapshot(encoded.subarray(offset, offset + 36));
    offset += 36;
    const endTreeOfHistoricPrivateDataTreeRootsSnapshot = bufferToAppendOnlyTreeSnapshot(
      encoded.subarray(offset, offset + 36),
    );
    offset += 36;
    const endTreeOfHistoricContractTreeRootsSnapshot = bufferToAppendOnlyTreeSnapshot(
      encoded.subarray(offset, offset + 36),
    );
    offset += 36;

    const newCommitments: Buffer[] = [];
    const newNullifiers: Buffer[] = [];
    const newContracts: Buffer[] = [];
    const newContractData: ContractData[] = [];

    const newCommitmentCount = encoded.readUInt32BE(offset);
    offset += 4;
    for (let i = 0; i < newCommitmentCount; i++) {
      newCommitments.push(encoded.subarray(offset, offset + 32));
      offset += 32;
    }
    const newNullifierCount = encoded.readUInt32BE(offset);
    offset += 4;
    for (let i = 0; i < newNullifierCount; i++) {
      newNullifiers.push(encoded.subarray(offset, offset + 32));
      offset += 32;
    }

    const newContractCount = encoded.readUInt32BE(offset);
    offset += 4;
    for (let i = 0; i < newContractCount; i++) {
      newContracts.push(encoded.subarray(offset, offset + 32));
      offset += 32;
    }
    for (let i = 0; i < newContractCount; i++) {
      newContractData.push(bufferToContractData(encoded.subarray(offset, offset + 52)));
      offset += 52;
    }

    return new L2Block(
      blockNum,
      startPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
      endPrivateDataTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
      newCommitments,
      newNullifiers,
      newContracts,
      newContractData,
    );
  }
}

/**
 * UTILITIES.
 */

/**
 * Serialize the contract data as a buffer.
 * @param contractData - The contract data to serialize.
 * @returns The serialized contract data as a buffer.
 */
export function contractDataToBuffer(contractData: ContractData): Buffer {
  return Buffer.concat([contractData.aztecAddress, contractData.ethAddress]);
}

/**
 * Deserialize the contract data from a buffer.
 * @param buffer - The buffer to deserialize.
 * @returns The contract data.
 */
export function bufferToContractData(buffer: Buffer): ContractData {
  return {
    aztecAddress: buffer.subarray(0, 32),
    ethAddress: buffer.subarray(32, 52),
  };
}

/**
 * Serialize the append only tree snapshot as a buffer.
 * @param snapshot - The snapshot to serialize.
 * @returns The serialized snapshot as a buffer.
 */
export function appendOnlyTreeSnapshotToBuffer(snapshot: AppendOnlyTreeSnapshot): Buffer {
  const buffer = Buffer.concat([numToUInt32BE(snapshot.nextAvailableLeafIndex), snapshot.root]);
  return buffer;
}

/**
 * Deserialize the append only tree snapshot from a buffer.
 * @param buffer - The buffer to deserialize.
 * @returns The append only tree snapshot.
 */
export function bufferToAppendOnlyTreeSnapshot(buffer: Buffer): AppendOnlyTreeSnapshot {
  const nextAvailableLeafIndex = buffer.readUInt32BE(0);
  const root = buffer.subarray(4);
  return {
    nextAvailableLeafIndex,
    root,
  };
}

/**
 * UNUSED TYPED THAT COULD BE USEFUL.
 */

/**
 * The fixed size data that makes up the rollup header.
 */
export type L2BlockCalldataHeader = {
  /**
   * The id of the rollup.
   * Similar to the block number in Ethereum.
   */
  rollupId: number;
  /**
   * The tree snapshot of the private data tree at the start of the rollup.
   */
  startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the nullifier tree at the start of the rollup.
   */
  startNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the contract tree at the start of the rollup.
   */
  startContractTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic private data tree roots at the start of the rollup.
   */
  startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic contract tree roots at the start of the rollup.
   */
  startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the private data tree at the end of the rollup.
   * By using start and end, we know the number of new commitments in the rollup.
   */
  endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the nullifier tree at the end of the rollup.
   * By using start and end, we know the number of new nullifiers in the rollup.
   */
  endNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the contract tree at the end of the rollup.
   */
  endContractTreeSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic private data tree roots at the end of the rollup.
   */
  endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot;
  /**
   * The tree snapshot of the historic contract tree roots at the end of the rollup.
   */
  endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot;
};

/**
 * The data that makes up the rollup calldata.
 */
export type L2BlockCalldata = {
  /**
   * The header of the rollup calldata.
   */
  header: L2BlockCalldataHeader;
  /**
   * The commitments to be inserted into the private data tree.
   * The commitments are field elements, and 4 commitments are inserted for each kernel proof.
   */
  newCommitments: Buffer[];
  /**
   * The nullifiers to be inserted into the nullifier tree.
   * The nullifiers are field elements, and 4 nullifiers are inserted for each kernel proof.
   */
  newNullifiers: Buffer[];
  /**
   * The contracts leafs to be inserted into the contract tree.
   * The contracts are field element, there can be at most 1 contract deployed for each kernel proof.
   */
  newContracts: Buffer[];
  /**
   * The aztec_address and eth_address for the deployed contract and its portal contract.
   * The aztec_address is the address of the deployed contract, will be a field element (32 bytes)
   * The eth_address will be the address of the portal contract, or address(0) if no portal is used (20 bytes).
   */
  newContractData: ContractData[];
};
