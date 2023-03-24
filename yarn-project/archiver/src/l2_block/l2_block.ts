import { AppendOnlyTreeSnapshot, EthAddress, Fr } from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';

/**
 * A contract data blob, containing L1 and L2 addresses.
 */
export class ContractData {
  constructor(
    /**
     * The L2 address of the contract, as a field element (32 bytes).
     */
    public aztecAddress: Fr,
    /**
     * The L1 address of the contract, (20 bytes).
     */
    public ethAddress: EthAddress,
  ) {}

  /**
   * Serializes this instance into a buffer, using 20 bytes for the eth address.
   * @returns Encoded buffer.
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.aztecAddress, this.ethAddress.buffer);
  }

  /**
   * Deserializes a contract data object from an encoded buffer, using 20 bytes for the eth address.
   * @param buffer - Byte array resulting from calling toBuffer.
   * @returns Deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ContractData(reader.readFr(), new EthAddress(reader.readBytes(EthAddress.SIZE_IN_BYTES)));
  }
}

/**
 * The data that makes up the rollup proof, with encoder decoder functions.
 * TODO: Reuse data types and serialization functions from circuits package.
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
    public newCommitments: Fr[],
    public newNullifiers: Fr[],
    public newContracts: Fr[],
    public newContractData: ContractData[],
  ) {}

  /**
   * Constructs a new instance from named fields.
   * @param fields - Fields to pass to the constructor.
   * @returns A new instance.
   */
  static fromFields(fields: {
    number: number;
    startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot;
    startNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
    startContractTreeSnapshot: AppendOnlyTreeSnapshot;
    startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot;
    startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot;
    endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot;
    endNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
    endContractTreeSnapshot: AppendOnlyTreeSnapshot;
    endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot;
    endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot;
    newCommitments: Fr[];
    newNullifiers: Fr[];
    newContracts: Fr[];
    newContractData: ContractData[];
  }) {
    return new this(
      fields.number,
      fields.startPrivateDataTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.startTreeOfHistoricContractTreeRootsSnapshot,
      fields.endPrivateDataTreeSnapshot,
      fields.endNullifierTreeSnapshot,
      fields.endContractTreeSnapshot,
      fields.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.endTreeOfHistoricContractTreeRootsSnapshot,
      fields.newCommitments,
      fields.newNullifiers,
      fields.newContracts,
      fields.newContractData,
    );
  }

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
    return serializeToBuffer(
      this.number,
      this.startPrivateDataTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      this.startTreeOfHistoricContractTreeRootsSnapshot,
      this.endPrivateDataTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      this.endTreeOfHistoricContractTreeRootsSnapshot,
      this.newCommitments.length,
      this.newCommitments,
      this.newNullifiers.length,
      this.newNullifiers,
      this.newContracts.length,
      this.newContracts,
      this.newContractData,
    );
  }

  /**
   * Alias for encode.
   * @returns The encoded L2 block data.
   */
  toBuffer() {
    return this.encode();
  }

  /**
   * Decode the L2 block data from a buffer.
   * @param encoded - The encoded L2 block data.
   * @returns The decoded L2 block data.
   */
  static decode(encoded: Buffer | BufferReader) {
    const reader = BufferReader.asReader(encoded);
    const number = reader.readNumber();
    const startPrivateDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startTreeOfHistoricPrivateDataTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startTreeOfHistoricContractTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endPrivateDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endTreeOfHistoricPrivateDataTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endTreeOfHistoricContractTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const newCommitments = reader.readVector(Fr);
    const newNullifiers = reader.readVector(Fr);
    const newContracts = reader.readVector(Fr);
    const newContractData = reader.readArray(newContracts.length, ContractData);

    return new L2Block(
      number,
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

  /**
   * Inspect for debugging purposes..
   * @param maxBufferSize - The number of bytes to be extracted from buffer.
   * @returns A human-friendly string representation of the l2block.
   */
  inspect(maxBufferSize = 4): string {
    const inspectTreeSnapshot = (s: AppendOnlyTreeSnapshot): string =>
      `(${s.nextAvailableLeafIndex}, 0x${s.root.toBuffer().subarray(0, maxBufferSize).toString('hex')})`;
    const inspectFrArray = (arr: Fr[]): string =>
      '[' + arr.map(fr => '0x' + fr.toBuffer().subarray(0, maxBufferSize).toString('hex')).join(', ') + ']';
    const inspectContractDataArray = (arr: ContractData[]): string =>
      '[' +
      arr
        .map(
          cd =>
            `(0x${cd.aztecAddress.toBuffer().subarray(0, maxBufferSize).toString('hex')}, 0x${cd.ethAddress.buffer
              .subarray(0, maxBufferSize)
              .toString('hex')})`,
        )
        .join(', ') +
      ']';
    return [
      `L2Block`,
      `number: ${this.number}`,
      `startPrivateDataTreeSnapshot: ${inspectTreeSnapshot(this.startPrivateDataTreeSnapshot)}`,
      `startNullifierTreeSnapshot: ${inspectTreeSnapshot(this.startNullifierTreeSnapshot)}`,
      `startContractTreeSnapshot: ${inspectTreeSnapshot(this.startContractTreeSnapshot)}`,
      `startTreeOfHistoricPrivateDataTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      )}`,
      `startTreeOfHistoricContractTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.startTreeOfHistoricContractTreeRootsSnapshot,
      )}`,
      `endPrivateDataTreeSnapshot: ${inspectTreeSnapshot(this.endPrivateDataTreeSnapshot)}`,
      `endNullifierTreeSnapshot: ${inspectTreeSnapshot(this.endNullifierTreeSnapshot)}`,
      `endContractTreeSnapshot: ${inspectTreeSnapshot(this.endContractTreeSnapshot)}`,
      `endTreeOfHistoricPrivateDataTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      )}`,
      `endTreeOfHistoricContractTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.endTreeOfHistoricContractTreeRootsSnapshot,
      )}`,
      `newCommitments: ${inspectFrArray(this.newCommitments)}`,
      `newNullifiers: ${inspectFrArray(this.newNullifiers)}`,
      `newContracts: ${inspectFrArray(this.newContracts)}`,
      `newContractData: ${inspectContractDataArray(this.newContractData)}`,
    ].join('\n');
  }
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
