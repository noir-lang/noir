import {
  AppendOnlyTreeSnapshot,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  KERNEL_NEW_L2_TO_L1_MSGS_LENGTH,
} from '@aztec/circuits.js';
import { makeAppendOnlyTreeSnapshot } from '@aztec/circuits.js/factories';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';
import { Fr } from '@aztec/foundation/fields';
import times from 'lodash.times';
import { ContractData } from './contract_data.js';
import { L2Tx } from './l2_tx.js';
import { PublicDataWrite } from './public_data_write.js';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { sha256 } from '@aztec/foundation/crypto';
/**
 * The data that makes up the rollup proof, with encoder decoder functions.
 * TODO: Reuse data types and serialization functions from circuits package.
 */
export class L2Block {
  constructor(
    /**
     * The number of the L2 block.
     */
    public number: number,
    /**
     * The tree snapshot of the private data tree at the start of the rollup.
     */
    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the nullifier tree at the start of the rollup.
     */
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the contract tree at the start of the rollup.
     */
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic private data tree roots at the start of the rollup.
     */
    public startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic contract tree roots at the start of the rollup.
     */
    public startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree root of the public data tree at the start of the rollup.
     */
    public startPublicDataTreeRoot: Fr,
    /**
     * The tree snapshot of the L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic L2 message tree roots at the start of the rollup.
     */
    public startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the private data tree at the end of the rollup.
     */
    public endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the nullifier tree at the end of the rollup.
     */
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the contract tree at the end of the rollup.
     */
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic private data tree roots at the end of the rollup.
     */
    public endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic contract tree roots at the end of the rollup.
     */
    public endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree root of the public data tree at the end of the rollup.
     */
    public endPublicDataTreeRoot: Fr,
    /**
     * The tree snapshot of the L2 message tree at the end of the rollup.
     */
    public endL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic L2 message tree roots at the end of the rollup.
     */
    public endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The commitments to be inserted into the private data tree.
     */
    public newCommitments: Fr[],
    /**
     * The nullifiers to be inserted into the nullifier tree.
     */
    public newNullifiers: Fr[],
    /**
     * The public data writes to be inserted into the public data tree.
     */
    public newPublicDataWrites: PublicDataWrite[],
    /**
     * The L2 to L1 messages to be inserted into the messagebox on L1.
     */
    public newL2ToL1Msgs: Fr[],
    /**
     * The contracts leafs to be inserted into the contract tree.
     */
    public newContracts: Fr[],
    /**
     * The aztec address and ethereum address for the deployed contract and its portal contract.
     */
    public newContractData: ContractData[],
    /**
     * The L1 to L2 messages to be inserted into the L2 toL2 message tree.
     */
    public newL1ToL2Messages: Fr[] = [],
  ) {}

  /**
   * Creates an L2 block containing random data.
   * @param l2BlockNum - The number of the L2 block.
   * @param txsPerBlock - The number of transactions to include in the block.
   * @returns The L2 block.
   */
  static random(l2BlockNum: number, txsPerBlock = 4): L2Block {
    const newNullifiers = times(KERNEL_NEW_NULLIFIERS_LENGTH * txsPerBlock, Fr.random);
    const newCommitments = times(KERNEL_NEW_COMMITMENTS_LENGTH * txsPerBlock, Fr.random);
    const newContracts = times(KERNEL_NEW_CONTRACTS_LENGTH * txsPerBlock, Fr.random);
    const newContractData = times(KERNEL_NEW_CONTRACTS_LENGTH * txsPerBlock, ContractData.random);
    const newPublicDataWrites = times(KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * txsPerBlock, PublicDataWrite.random);
    const newL1ToL2Messages = times(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, Fr.random);
    const newL2ToL1Msgs = times(KERNEL_NEW_L2_TO_L1_MSGS_LENGTH, Fr.random);

    return L2Block.fromFields({
      number: l2BlockNum,
      startPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startContractTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startPublicDataTreeRoot: Fr.random(),
      startL1ToL2MessageTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(0),
      startTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(0),
      startTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(0),
      endPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot(newCommitments.length),
      endNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(newNullifiers.length),
      endContractTreeSnapshot: makeAppendOnlyTreeSnapshot(newContracts.length),
      endPublicDataTreeRoot: Fr.random(),
      endL1ToL2MessageTreeSnapshot: makeAppendOnlyTreeSnapshot(1),
      endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(1),
      endTreeOfHistoricPrivateDataTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(1),
      endTreeOfHistoricContractTreeRootsSnapshot: makeAppendOnlyTreeSnapshot(1),
      newCommitments,
      newNullifiers,
      newContracts,
      newContractData,
      newPublicDataWrites,
      newL1ToL2Messages,
      newL2ToL1Msgs,
    });
  }

  /**
   * Constructs a new instance from named fields.
   * @param fields - Fields to pass to the constructor.
   * @returns A new instance.
   */
  static fromFields(fields: {
    /**
     * The number of the L2 block.
     */
    number: number;
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
     * The tree root of the public data tree at the start of the rollup.
     */
    startPublicDataTreeRoot: Fr;
    /**
     * The tree snapshot of the L2 message tree at the start of the rollup.
     */
    startL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot;
    /**
     * The tree snapshot of the historic L2 message tree roots at the start of the rollup.
     */
    startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot;
    /**
     * The tree snapshot of the private data tree at the end of the rollup.
     */
    endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot;
    /**
     * The tree snapshot of the nullifier tree at the end of the rollup.
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
    /**
     * The tree root of the public data tree at the end of the rollup.
     */
    endPublicDataTreeRoot: Fr;
    /**
     * The tree snapshot of the L2 message tree at the end of the rollup.
     */
    endL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot;
    /**
     * The tree snapshot of the historic L2 message tree roots at the end of the rollup.
     */
    endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot;
    /**
     * The commitments to be inserted into the private data tree.
     */
    newCommitments: Fr[];
    /**
     * The nullifiers to be inserted into the nullifier tree.
     */
    newNullifiers: Fr[];
    /**
     * The public data writes to be inserted into the public data tree.
     */
    newPublicDataWrites: PublicDataWrite[];
    /**
     * The L2 to L1 messages to be inserted into the messagebox on L1.
     */
    newL2ToL1Msgs: Fr[];
    /**
     * The contracts leafs to be inserted into the contract tree.
     */
    newContracts: Fr[];
    /**
     * The aztec address and ethereum address for the deployed contract and its portal contract.
     */
    newContractData: ContractData[];
    /**
     * The L1 to L2 messages to be inserted into the L2 toL2 message tree.
     */
    newL1ToL2Messages: Fr[];
  }) {
    return new this(
      fields.number,
      fields.startPrivateDataTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.startTreeOfHistoricContractTreeRootsSnapshot,
      fields.startPublicDataTreeRoot,
      fields.startL1ToL2MessageTreeSnapshot,
      fields.startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      fields.endPrivateDataTreeSnapshot,
      fields.endNullifierTreeSnapshot,
      fields.endContractTreeSnapshot,
      fields.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.endTreeOfHistoricContractTreeRootsSnapshot,
      fields.endPublicDataTreeRoot,
      fields.endL1ToL2MessageTreeSnapshot,
      fields.endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      fields.newCommitments,
      fields.newNullifiers,
      fields.newPublicDataWrites,
      fields.newL2ToL1Msgs,
      fields.newContracts,
      fields.newContractData,
      fields.newL1ToL2Messages,
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
      this.startPublicDataTreeRoot,
      this.startL1ToL2MessageTreeSnapshot,
      this.startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      this.endPrivateDataTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      this.endTreeOfHistoricContractTreeRootsSnapshot,
      this.endPublicDataTreeRoot,
      this.endL1ToL2MessageTreeSnapshot,
      this.endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      this.newCommitments.length,
      this.newCommitments,
      this.newNullifiers.length,
      this.newNullifiers,
      this.newPublicDataWrites.length,
      this.newPublicDataWrites,
      this.newL2ToL1Msgs.length,
      this.newL2ToL1Msgs,
      this.newContracts.length,
      this.newContracts,
      this.newContractData,
      this.newL1ToL2Messages.length,
      this.newL1ToL2Messages,
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
    const startPublicDataTreeRoot = reader.readObject(Fr);
    const startL1ToL2MessageTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endPrivateDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endTreeOfHistoricPrivateDataTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endTreeOfHistoricContractTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endPublicDataTreeRoot = reader.readObject(Fr);
    const endL1ToL2MessageTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const newCommitments = reader.readVector(Fr);
    const newNullifiers = reader.readVector(Fr);
    const newPublicDataWrites = reader.readVector(PublicDataWrite);
    const newL2ToL1Msgs = reader.readVector(Fr);
    const newContracts = reader.readVector(Fr);
    const newContractData = reader.readArray(newContracts.length, ContractData);
    // TODO(sean): could an optimisation of this be that it is encoded such that zeros are assumed
    const newL1ToL2Messages = reader.readVector(Fr);

    return L2Block.fromFields({
      number,
      startPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
      startPublicDataTreeRoot,
      startL1ToL2MessageTreeSnapshot,
      startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      endPrivateDataTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
      endPublicDataTreeRoot,
      endL1ToL2MessageTreeSnapshot,
      endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      newCommitments,
      newNullifiers,
      newPublicDataWrites,
      newL2ToL1Msgs,
      newContracts,
      newContractData,
      newL1ToL2Messages,
    });
  }

  /**
   * Computes the public inputs hash for the L2 block.
   * The same output as the hash of RootRollupPublicInputs.
   * @returns The public input hash for the L2 block as a field element.
   */
  getPublicInputsHash(): Fr {
    const buf = serializeToBuffer(
      this.startPrivateDataTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      this.startTreeOfHistoricContractTreeRootsSnapshot,
      this.startPublicDataTreeRoot,
      this.startL1ToL2MessageTreeSnapshot,
      this.startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      this.endPrivateDataTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      this.endTreeOfHistoricContractTreeRootsSnapshot,
      this.endPublicDataTreeRoot,
      this.endL1ToL2MessageTreeSnapshot,
      this.endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      this.getCalldataHash(),
      this.getL1ToL2MessagesHash(),
    );

    const temp = toBigIntBE(sha256(buf));
    return Fr.fromBuffer(toBufferBE(temp % Fr.MODULUS, 32));
  }

  /**
   * Computes the start state hash (should equal contract data before block).
   * @returns The start state hash for the L2 block.
   */
  getStartStateHash() {
    const inputValue = serializeToBuffer(
      this.number - 1,
      this.startPrivateDataTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      this.startTreeOfHistoricContractTreeRootsSnapshot,
      this.startPublicDataTreeRoot,
      this.startL1ToL2MessageTreeSnapshot,
      this.startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
    );
    return sha256(inputValue);
  }

  /**
   * Computes the end state hash (should equal contract data after block).
   * @returns The end state hash for the L2 block.
   */
  getEndStateHash() {
    const inputValue = serializeToBuffer(
      this.number,
      this.endPrivateDataTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      this.endTreeOfHistoricContractTreeRootsSnapshot,
      this.endPublicDataTreeRoot,
      this.endL1ToL2MessageTreeSnapshot,
      this.endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
    );
    return sha256(inputValue);
  }

  /**
   * Computes the calldata hash for the L2 block
   * This calldata hash is also computed by the rollup contract when the block is submitted,
   * and inside the circuit, it is part of the public inputs.
   * @returns The calldata hash.
   */
  getCalldataHash() {
    const computeRoot = (leafs: Buffer[]): Buffer => {
      const layers: Buffer[][] = [leafs];
      let activeLayer = 0;

      while (layers[activeLayer].length > 1) {
        const layer: Buffer[] = [];
        const layerLength = layers[activeLayer].length;

        for (let i = 0; i < layerLength; i += 2) {
          const left = layers[activeLayer][i];
          const right = layers[activeLayer][i + 1];

          layer.push(sha256(Buffer.concat([left, right])));
        }

        layers.push(layer);
        activeLayer++;
      }

      return layers[layers.length - 1][0];
    };

    const leafCount = this.newCommitments.length / (KERNEL_NEW_COMMITMENTS_LENGTH * 2);
    const leafs: Buffer[] = [];

    for (let i = 0; i < leafCount; i++) {
      const commitmentPerBase = KERNEL_NEW_COMMITMENTS_LENGTH * 2;
      const nullifierPerBase = KERNEL_NEW_NULLIFIERS_LENGTH * 2;
      const publicDataWritesPerBase = KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * 2; // @note why is this constant named differently?
      const l2ToL1MsgsPerBase = KERNEL_NEW_L2_TO_L1_MSGS_LENGTH * 2;
      const commitmentBuffer = Buffer.concat(
        this.newCommitments.slice(i * commitmentPerBase, (i + 1) * commitmentPerBase).map(x => x.toBuffer()),
      );
      const nullifierBuffer = Buffer.concat(
        this.newNullifiers.slice(i * nullifierPerBase, (i + 1) * nullifierPerBase).map(x => x.toBuffer()),
      );
      const dataWritesBuffer = Buffer.concat(
        this.newPublicDataWrites
          .slice(i * publicDataWritesPerBase, (i + 1) * publicDataWritesPerBase)
          .map(x => x.toBuffer()),
      );
      const newL2ToL1MsgsBuffer = Buffer.concat(
        this.newL2ToL1Msgs.slice(i * l2ToL1MsgsPerBase, (i + 1) * l2ToL1MsgsPerBase).map(x => x.toBuffer()),
      );

      const inputValue = Buffer.concat([
        commitmentBuffer,
        nullifierBuffer,
        dataWritesBuffer,
        newL2ToL1MsgsBuffer,
        this.newContracts[i * 2].toBuffer(),
        this.newContracts[i * 2 + 1].toBuffer(),
        this.newContractData[i * 2].contractAddress.toBuffer(),
        this.newContractData[i * 2].portalContractAddress.toBuffer32(),
        this.newContractData[i * 2 + 1].contractAddress.toBuffer(),
        this.newContractData[i * 2 + 1].portalContractAddress.toBuffer32(),
      ]);
      leafs.push(sha256(inputValue));
    }
    return computeRoot(leafs);
  }

  /**
   * Compute the hash of all of this blocks l1 to l2 messages,
   * The hash is also calculated within the contract when the block is submitted.
   * @returns The hash of all of the l1 to l2 messages.
   */
  getL1ToL2MessagesHash(): Buffer {
    // Create a long buffer of all of the l1 to l2 messages
    const l1ToL2Messages = Buffer.concat(this.newL1ToL2Messages.map(message => message.toBuffer()));
    return sha256(l1ToL2Messages);
  }

  /**
   * Get the ith transaction in an L2 block.
   * @param txIndex - The index of the tx in the block.
   * @returns The tx.
   */
  getTx(txIndex: number) {
    const numTxs = Math.floor(this.newCommitments.length / KERNEL_NEW_COMMITMENTS_LENGTH);
    if (txIndex >= numTxs) {
      throw new Error(`Failed to get tx ${txIndex}. Block ${this.number} only has ${numTxs} txs.`);
    }

    const newCommitments = this.newCommitments.slice(
      KERNEL_NEW_COMMITMENTS_LENGTH * txIndex,
      KERNEL_NEW_COMMITMENTS_LENGTH * (txIndex + 1),
    );
    const newNullifiers = this.newNullifiers.slice(
      KERNEL_NEW_NULLIFIERS_LENGTH * txIndex,
      KERNEL_NEW_NULLIFIERS_LENGTH * (txIndex + 1),
    );
    const newPublicDataWrites = this.newPublicDataWrites.slice(
      KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * txIndex,
      KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH * (txIndex + 1),
    );
    const newL2ToL1Msgs = this.newL2ToL1Msgs.slice(
      KERNEL_NEW_L2_TO_L1_MSGS_LENGTH * txIndex,
      KERNEL_NEW_L2_TO_L1_MSGS_LENGTH * (txIndex + 1),
    );
    const newContracts = this.newContracts.slice(
      KERNEL_NEW_CONTRACTS_LENGTH * txIndex,
      KERNEL_NEW_CONTRACTS_LENGTH * (txIndex + 1),
    );
    const newContractData = this.newContractData.slice(
      KERNEL_NEW_CONTRACTS_LENGTH * txIndex,
      KERNEL_NEW_CONTRACTS_LENGTH * (txIndex + 1),
    );

    return new L2Tx(newCommitments, newNullifiers, newPublicDataWrites, newL2ToL1Msgs, newContracts, newContractData);
  }

  /**
   * Inspect for debugging purposes..
   * @param maxBufferSize - The number of bytes to be extracted from buffer.
   * @returns A human-friendly string representation of the l2Block.
   */
  inspect(maxBufferSize = 4): string {
    const inspectHex = (fr: {
      /**
       * A function used to serialize the field element to a buffer.
       */
      toBuffer: () => Buffer;
    }): string => `0x${fr.toBuffer().subarray(0, maxBufferSize).toString('hex')}`;
    const inspectArray = <T>(arr: T[], inspector: (t: T) => string) => '[' + arr.map(inspector).join(', ') + ']';

    const inspectTreeSnapshot = (s: AppendOnlyTreeSnapshot): string =>
      `(${s.nextAvailableLeafIndex}, ${inspectHex(s.root)})`;
    const inspectFrArray = (arr: Fr[]): string => inspectArray(arr, inspectHex);
    const inspectContractDataArray = (arr: ContractData[]): string =>
      inspectArray(arr, cd => `(${inspectHex(cd.contractAddress)}, ${inspectHex(cd.portalContractAddress)})`);
    const inspectPublicDataWriteArray = (arr: PublicDataWrite[]): string =>
      inspectArray(arr, pdw => `(${inspectHex(pdw.leafIndex)}, ${inspectHex(pdw.newValue)})`);

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
      `startPublicDataTreeRoot: ${this.startPublicDataTreeRoot.toString()}`,
      `startL1ToL2MessageTreeSnapshot: ${inspectTreeSnapshot(this.startL1ToL2MessageTreeSnapshot)}`,
      `startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      )}`,
      `endPrivateDataTreeSnapshot: ${inspectTreeSnapshot(this.endPrivateDataTreeSnapshot)}`,
      `endNullifierTreeSnapshot: ${inspectTreeSnapshot(this.endNullifierTreeSnapshot)}`,
      `endContractTreeSnapshot: ${inspectTreeSnapshot(this.endContractTreeSnapshot)}`,
      `endPublicDataTreeRoot: ${this.endPublicDataTreeRoot.toString()}`,
      `endTreeOfHistoricPrivateDataTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      )}`,
      `endTreeOfHistoricContractTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.endTreeOfHistoricContractTreeRootsSnapshot,
      )}`,
      `endPublicDataTreeRoot: ${this.endPublicDataTreeRoot.toString()}`,
      `endL1ToL2MessageTreeSnapshot: ${inspectTreeSnapshot(this.endL1ToL2MessageTreeSnapshot)}`,
      `endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: ${inspectTreeSnapshot(
        this.endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      )}`,
      `newCommitments: ${inspectFrArray(this.newCommitments)}`,
      `newNullifiers: ${inspectFrArray(this.newNullifiers)}`,
      `newPublicDataWrite: ${inspectPublicDataWriteArray(this.newPublicDataWrites)}`,
      `newL2ToL1Msgs: ${inspectFrArray(this.newL2ToL1Msgs)}`,
      `newContracts: ${inspectFrArray(this.newContracts)}`,
      `newContractData: ${inspectContractDataArray(this.newContractData)}`,
      `newPublicDataWrite: ${inspectPublicDataWriteArray(this.newPublicDataWrites)}`,
      `newL1ToL2Messages: ${inspectFrArray(this.newL1ToL2Messages)}`,
    ].join('\n');
  }
}
