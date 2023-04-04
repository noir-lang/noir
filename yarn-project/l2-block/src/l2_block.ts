import { Fr } from '@aztec/foundation';
import {
  AppendOnlyTreeSnapshot,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
} from '@aztec/circuits.js';
import { makeAppendOnlyTreeSnapshot } from '@aztec/circuits.js/factories';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';
import { ContractData } from './contract_data.js';

/**
 * The data that makes up the rollup proof, with encoder decoder functions.
 * TODO: Reuse data types and serialization functions from circuits package.
 */
export class L2Block {
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

  static random(l2BlockNum: number, txsPerBlock = 1) {
    const newNullifiers = Array(KERNEL_NEW_NULLIFIERS_LENGTH * txsPerBlock)
      .fill(0)
      .map(() => Fr.random());
    const newCommitments = Array(KERNEL_NEW_COMMITMENTS_LENGTH * txsPerBlock)
      .fill(0)
      .map(() => Fr.random());
    const newContracts = Array(KERNEL_NEW_CONTRACTS_LENGTH * txsPerBlock)
      .fill(0)
      .map(() => Fr.random());
    const newContractsData = Array(KERNEL_NEW_CONTRACTS_LENGTH * txsPerBlock)
      .fill(0)
      .map(() => ContractData.random());

    return new L2Block(
      l2BlockNum,
      makeAppendOnlyTreeSnapshot(0),
      makeAppendOnlyTreeSnapshot(0),
      makeAppendOnlyTreeSnapshot(0),
      makeAppendOnlyTreeSnapshot(0),
      makeAppendOnlyTreeSnapshot(0),
      makeAppendOnlyTreeSnapshot(newCommitments.length),
      makeAppendOnlyTreeSnapshot(newNullifiers.length),
      makeAppendOnlyTreeSnapshot(newContracts.length),
      makeAppendOnlyTreeSnapshot(1),
      makeAppendOnlyTreeSnapshot(1),
      newCommitments,
      newNullifiers,
      newContracts,
      newContractsData,
    );
  }

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
   * @returns A human-friendly string representation of the l2Block.
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
            `(0x${cd.aztecAddress.toBuffer().subarray(0, maxBufferSize).toString('hex')}, 0x${cd.ethAddress
              .toBuffer()
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
