import {
  AppendOnlyTreeSnapshot,
  GlobalVariables,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  STRING_ENCODING,
} from '@aztec/circuits.js';
import { makeAppendOnlyTreeSnapshot, makeGlobalVariables } from '@aztec/circuits.js/factories';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';
import { keccak, sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import times from 'lodash.times';

import { ContractData } from './contract_data.js';
import { L2Tx } from './l2_tx.js';
import { LogType, TxL2Logs } from './logs/index.js';
import { L2BlockL2Logs } from './logs/l2_block_l2_logs.js';
import { PublicDataWrite } from './public_data_write.js';

/**
 * The data that makes up the rollup proof, with encoder decoder functions.
 * TODO: Reuse data types and serialization functions from circuits package.
 */
export class L2Block {
  /* Having logger static to avoid issues with comparing 2 block */
  private static logger = createDebugLogger('aztec:l2_block');

  /**
   * The number of L2Tx in this L2Block.
   */
  public numberOfTxs: number;

  /**
   * Encrypted logs emitted by txs in this block.
   * @remarks `L2BlockL2Logs.txLogs` array has to match number of txs in this block and has to be in the same order
   *          (e.g. logs from the first tx on the first place...).
   * @remarks Only private function can emit encrypted logs and for this reason length of
   *          `newEncryptedLogs.txLogs.functionLogs` is equal to the number of private function invocations in the tx.
   */
  public newEncryptedLogs?: L2BlockL2Logs;

  /**
   * Unencrypted logs emitted by txs in this block.
   * @remarks `L2BlockL2Logs.txLogs` array has to match number of txs in this block and has to be in the same order
   *          (e.g. logs from the first tx on the first place...).
   * @remarks Both private and public functions can emit unencrypted logs and for this reason length of
   *          `newUnencryptedLogs.txLogs.functionLogs` is equal to the number of all function invocations in the tx.
   */
  public newUnencryptedLogs?: L2BlockL2Logs;

  #l1BlockNumber?: bigint;

  constructor(
    /**
     * The number of the L2 block.
     */
    public number: number,
    /**
     * The global variables for the L2 block.
     */
    public globalVariables: GlobalVariables,
    /**
     * The tree snapshot of the note hash tree at the start of the rollup.
     */
    public startNoteHashTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the nullifier tree at the start of the rollup.
     */
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the contract tree at the start of the rollup.
     */
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the public data tree at the start of the rollup.
     */
    public startPublicDataTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the archive at the start of the rollup.
     */
    public startArchiveSnapshot: AppendOnlyTreeSnapshot = AppendOnlyTreeSnapshot.empty(),
    /**
     * The tree snapshot of the note hash tree at the end of the rollup.
     */
    public endNoteHashTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the nullifier tree at the end of the rollup.
     */
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the contract tree at the end of the rollup.
     */
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the public data tree at the end of the rollup.
     */
    public endPublicDataTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the L2 message tree at the end of the rollup.
     */
    public endL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the archive at the end of the rollup.
     */
    public endArchiveSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The commitments to be inserted into the note hash tree.
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
    newEncryptedLogs?: L2BlockL2Logs,
    newUnencryptedLogs?: L2BlockL2Logs,
    private blockHash?: Buffer,
    l1BlockNumber?: bigint,
  ) {
    if (newCommitments.length % MAX_NEW_COMMITMENTS_PER_TX !== 0) {
      throw new Error(`The number of new commitments must be a multiple of ${MAX_NEW_COMMITMENTS_PER_TX}.`);
    }

    if (newEncryptedLogs) {
      this.attachLogs(newEncryptedLogs, LogType.ENCRYPTED);
    }
    if (newUnencryptedLogs) {
      this.attachLogs(newUnencryptedLogs, LogType.UNENCRYPTED);
    }

    // Since the block is padded to always contain a fixed number of nullifiers we get number of txs by counting number
    // of non-zero tx hashes --> tx hash is set to be the first nullifier in the tx.
    this.numberOfTxs = 0;
    for (let i = 0; i < this.newNullifiers.length; i += MAX_NEW_NULLIFIERS_PER_TX) {
      if (!this.newNullifiers[i].equals(Fr.ZERO)) {
        this.numberOfTxs++;
      }
    }

    this.#l1BlockNumber = l1BlockNumber;
  }

  /**
   * Creates an L2 block containing random data.
   * @param l2BlockNum - The number of the L2 block.
   * @param txsPerBlock - The number of transactions to include in the block.
   * @param numPrivateCallsPerTx - The number of private function calls to include in each transaction.
   * @param numPublicCallsPerTx - The number of public function calls to include in each transaction.
   * @param numEncryptedLogsPerCall - The number of encrypted logs per 1 private function invocation.
   * @param numUnencryptedLogsPerCall - The number of unencrypted logs per 1 public function invocation.
   * @returns The L2 block.
   */
  static random(
    l2BlockNum: number,
    txsPerBlock = 4,
    numPrivateCallsPerTx = 2,
    numPublicCallsPerTx = 3,
    numEncryptedLogsPerCall = 2,
    numUnencryptedLogsPerCall = 1,
  ): L2Block {
    const newNullifiers = times(MAX_NEW_NULLIFIERS_PER_TX * txsPerBlock, Fr.random);
    const newCommitments = times(MAX_NEW_COMMITMENTS_PER_TX * txsPerBlock, Fr.random);
    const newContracts = times(MAX_NEW_CONTRACTS_PER_TX * txsPerBlock, Fr.random);
    const newContractData = times(MAX_NEW_CONTRACTS_PER_TX * txsPerBlock, ContractData.random);
    const newPublicDataWrites = times(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * txsPerBlock, PublicDataWrite.random);
    const newL1ToL2Messages = times(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, Fr.random);
    const newL2ToL1Msgs = times(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.random);
    const newEncryptedLogs = L2BlockL2Logs.random(
      txsPerBlock,
      numPrivateCallsPerTx,
      numEncryptedLogsPerCall,
      LogType.ENCRYPTED,
    );
    const newUnencryptedLogs = L2BlockL2Logs.random(
      txsPerBlock,
      numPublicCallsPerTx,
      numUnencryptedLogsPerCall,
      LogType.UNENCRYPTED,
    );

    return L2Block.fromFields(
      {
        number: l2BlockNum,
        globalVariables: makeGlobalVariables(0, l2BlockNum),
        startNoteHashTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
        startNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
        startContractTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
        startPublicDataTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
        startL1ToL2MessagesTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
        startArchiveSnapshot: makeAppendOnlyTreeSnapshot(0),
        endNoteHashTreeSnapshot: makeAppendOnlyTreeSnapshot(newCommitments.length),
        endNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(newNullifiers.length),
        endContractTreeSnapshot: makeAppendOnlyTreeSnapshot(newContracts.length),
        endPublicDataTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
        endL1ToL2MessagesTreeSnapshot: makeAppendOnlyTreeSnapshot(1),
        endArchiveSnapshot: makeAppendOnlyTreeSnapshot(1),
        newCommitments,
        newNullifiers,
        newContracts,
        newContractData,
        newPublicDataWrites,
        newL1ToL2Messages,
        newL2ToL1Msgs,
        newEncryptedLogs,
        newUnencryptedLogs,
      },
      undefined,
      // just for testing purposes, each random L2 block got emitted in the equivalent L1 block
      BigInt(l2BlockNum),
    );
  }

  /**
   * Constructs a new instance from named fields.
   * @param fields - Fields to pass to the constructor.
   * @param blockHash - Hash of the block.
   * @param l1BlockNumber - The block number of the L1 block that contains this L2 block.
   * @returns A new instance.
   */
  static fromFields(
    fields: {
      /**
       * The number of the L2 block.
       */
      number: number;
      /**
       * The global variables of the L2 block.
       */
      globalVariables: GlobalVariables;
      /**
       * The tree snapshot of the note hash tree at the start of the rollup.
       */
      startNoteHashTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the nullifier tree at the start of the rollup.
       */
      startNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the contract tree at the start of the rollup.
       */
      startContractTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the public data tree at the start of the rollup.
       */
      startPublicDataTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the L2 message tree at the start of the rollup.
       */
      startL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the archive at the start of the rollup.
       */
      startArchiveSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the note hash tree at the end of the rollup.
       */
      endNoteHashTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the nullifier tree at the end of the rollup.
       */
      endNullifierTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the contract tree at the end of the rollup.
       */
      endContractTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the public data tree at the end of the rollup.
       */
      endPublicDataTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the L2 message tree at the end of the rollup.
       */
      endL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the archive at the end of the rollup.
       */
      endArchiveSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The commitments to be inserted into the note hash tree.
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
      /**
       * Encrypted logs emitted by txs in a block.
       */
      newEncryptedLogs?: L2BlockL2Logs;
      /**
       * Unencrypted logs emitted by txs in a block.
       */
      newUnencryptedLogs?: L2BlockL2Logs;
    },
    blockHash?: Buffer,
    l1BlockNumber?: bigint,
  ) {
    return new this(
      fields.number,
      fields.globalVariables,
      fields.startNoteHashTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.startPublicDataTreeSnapshot,
      fields.startL1ToL2MessagesTreeSnapshot,
      fields.startArchiveSnapshot,
      fields.endNoteHashTreeSnapshot,
      fields.endNullifierTreeSnapshot,
      fields.endContractTreeSnapshot,
      fields.endPublicDataTreeSnapshot,
      fields.endL1ToL2MessagesTreeSnapshot,
      fields.endArchiveSnapshot,
      fields.newCommitments,
      fields.newNullifiers,
      fields.newPublicDataWrites,
      fields.newL2ToL1Msgs,
      fields.newContracts,
      fields.newContractData,
      fields.newL1ToL2Messages,
      fields.newEncryptedLogs,
      fields.newUnencryptedLogs,
      blockHash,
      l1BlockNumber,
    );
  }

  /**
   * Serializes a block without logs to a buffer.
   * @remarks This is used when the block is being served via JSON-RPC because the logs are expected to be served
   * separately.
   * @returns A serialized L2 block without logs.
   */
  toBuffer() {
    return serializeToBuffer(
      this.globalVariables,
      this.startNoteHashTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startPublicDataTreeSnapshot,
      this.startL1ToL2MessagesTreeSnapshot,
      this.startArchiveSnapshot,
      this.endNoteHashTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endPublicDataTreeSnapshot,
      this.endL1ToL2MessagesTreeSnapshot,
      this.endArchiveSnapshot,
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
   * Serializes a block with logs to a buffer.
   * @remarks This is used when the block is being submitted on L1.
   * @returns A serialized L2 block with logs.
   */
  toBufferWithLogs(): Buffer {
    if (this.newEncryptedLogs === undefined || this.newUnencryptedLogs === undefined) {
      throw new Error(
        `newEncryptedLogs and newUnencryptedLogs must be defined when encoding L2BlockData (block ${this.number})`,
      );
    }

    return serializeToBuffer(this.toBuffer(), this.newEncryptedLogs, this.newUnencryptedLogs);
  }

  /**
   * Serializes a block without logs to a string.
   * @remarks This is used when the block is being served via JSON-RPC because the logs are expected to be served
   * separately.
   * @returns A serialized L2 block without logs.
   */
  toString(): string {
    return this.toBuffer().toString(STRING_ENCODING);
  }

  /**
   * Deserializes L2 block without logs from a buffer.
   * @param buf - A serialized L2 block.
   * @returns Deserialized L2 block.
   */
  static fromBuffer(buf: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buf);
    const globalVariables = reader.readObject(GlobalVariables);
    const number = Number(globalVariables.blockNumber.toBigInt());
    const startNoteHashTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startPublicDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startL1ToL2MessagesTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startArchiveSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endNoteHashTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endPublicDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endL1ToL2MessagesTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endArchiveSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const newCommitments = reader.readVector(Fr);
    const newNullifiers = reader.readVector(Fr);
    const newPublicDataWrites = reader.readVector(PublicDataWrite);
    const newL2ToL1Msgs = reader.readVector(Fr);
    const newContracts = reader.readVector(Fr);
    const newContractData = reader.readArray(newContracts.length, ContractData);
    // TODO(sean): could an optimization of this be that it is encoded such that zeros are assumed
    const newL1ToL2Messages = reader.readVector(Fr);

    return L2Block.fromFields({
      number,
      globalVariables,
      startNoteHashTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startPublicDataTreeSnapshot,
      startL1ToL2MessagesTreeSnapshot: startL1ToL2MessagesTreeSnapshot,
      startArchiveSnapshot,
      endNoteHashTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endPublicDataTreeSnapshot,
      endL1ToL2MessagesTreeSnapshot,
      endArchiveSnapshot,
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
   * Deserializes L2 block with logs from a buffer.
   * @param buf - A serialized L2 block.
   * @returns Deserialized L2 block.
   */
  static fromBufferWithLogs(buf: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buf);
    const block = L2Block.fromBuffer(reader);
    const newEncryptedLogs = reader.readObject(L2BlockL2Logs);
    const newUnencryptedLogs = reader.readObject(L2BlockL2Logs);

    block.attachLogs(newEncryptedLogs, LogType.ENCRYPTED);
    block.attachLogs(newUnencryptedLogs, LogType.UNENCRYPTED);

    return block;
  }

  /**
   * Deserializes L2 block without logs from a buffer.
   * @param str - A serialized L2 block.
   * @returns Deserialized L2 block.
   */
  static fromString(str: string): L2Block {
    return L2Block.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  /**
   * Helper function to attach logs related to a block.
   * @param logs - The logs to be attached to a block.
   * @param logType - The type of logs to be attached.
   * @remarks Here, because we can have L2 blocks without logs and those logs can be attached later.
   */
  attachLogs(logs: L2BlockL2Logs, logType: LogType) {
    const logFieldName = logType === LogType.ENCRYPTED ? 'newEncryptedLogs' : 'newUnencryptedLogs';

    if (this[logFieldName]) {
      if (this[logFieldName]?.equals(logs)) {
        L2Block.logger(`${logFieldName} logs already attached`);
        return;
      }
      throw new Error(`Trying to attach different ${logFieldName} logs to block ${this.number}.`);
    }

    L2Block.logger(`Attaching ${logFieldName} ${logs.getTotalLogCount()} logs to block ${this.number}`);

    const numTxs = this.newCommitments.length / MAX_NEW_COMMITMENTS_PER_TX;

    if (numTxs !== logs.txLogs.length) {
      throw new Error(
        `Number of txLogs within ${logFieldName} does not match number of transactions. Expected: ${numTxs} Got: ${logs.txLogs.length}`,
      );
    }

    this[logFieldName] = logs;
  }

  /**
   * Sets the L1 block number that included this block
   * @param l1BlockNumber - The block number of the L1 block that contains this L2 block.
   */
  public setL1BlockNumber(l1BlockNumber: bigint) {
    this.#l1BlockNumber = l1BlockNumber;
  }

  /**
   * Gets the L1 block number that included this block
   */
  public getL1BlockNumber(): bigint {
    if (typeof this.#l1BlockNumber === 'undefined') {
      throw new Error('L1 block number has to be attached before calling "getL1BlockNumber"');
    }

    return this.#l1BlockNumber;
  }

  /**
   * Returns the block's hash.
   * @returns The block's hash.
   */
  public getBlockHash(): Buffer {
    if (!this.blockHash) {
      this.blockHash = keccak(this.toBufferWithLogs());
    }
    return this.blockHash;
  }

  /**
   * Computes the public inputs hash for the L2 block.
   * The same output as the hash of RootRollupPublicInputs.
   * @returns The public input hash for the L2 block as a field element.
   */
  getPublicInputsHash(): Fr {
    const buf = serializeToBuffer(
      this.globalVariables,
      this.startNoteHashTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startPublicDataTreeSnapshot,
      this.startL1ToL2MessagesTreeSnapshot,
      this.startArchiveSnapshot,
      this.endNoteHashTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endPublicDataTreeSnapshot,
      this.endL1ToL2MessagesTreeSnapshot,
      this.endArchiveSnapshot,
      this.getCalldataHash(),
      this.getL1ToL2MessagesHash(),
    );

    return Fr.fromBufferReduce(sha256(buf));
  }

  /**
   * Computes the start state hash (should equal contract data before block).
   * @returns The start state hash for the L2 block.
   */
  getStartStateHash() {
    const inputValue = serializeToBuffer(
      new Fr(this.number - 1),
      this.startNoteHashTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startPublicDataTreeSnapshot,
      this.startL1ToL2MessagesTreeSnapshot,
      this.startArchiveSnapshot,
    );
    return sha256(inputValue);
  }

  /**
   * Computes the end state hash (should equal contract data after block).
   * @returns The end state hash for the L2 block.
   */
  getEndStateHash() {
    const inputValue = serializeToBuffer(
      this.globalVariables.blockNumber,
      this.endNoteHashTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endPublicDataTreeSnapshot,
      this.endL1ToL2MessagesTreeSnapshot,
      this.endArchiveSnapshot,
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
    if (this.newEncryptedLogs === undefined) {
      throw new Error('Encrypted logs has to be attached before calling "getCalldataHash"');
    }

    if (this.newUnencryptedLogs === undefined) {
      throw new Error('Unencrypted logs has to be attached before calling "getCalldataHash"');
    }

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

    const leafCount = this.newCommitments.length / (MAX_NEW_COMMITMENTS_PER_TX * 2);
    const leafs: Buffer[] = [];

    for (let i = 0; i < leafCount; i++) {
      const commitmentsPerBase = MAX_NEW_COMMITMENTS_PER_TX * 2;
      const nullifiersPerBase = MAX_NEW_NULLIFIERS_PER_TX * 2;
      const publicDataUpdateRequestsPerBase = MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2;
      const l2ToL1MsgsPerBase = MAX_NEW_L2_TO_L1_MSGS_PER_TX * 2;
      const commitmentsBuffer = Buffer.concat(
        this.newCommitments.slice(i * commitmentsPerBase, (i + 1) * commitmentsPerBase).map(x => x.toBuffer()),
      );
      const nullifiersBuffer = Buffer.concat(
        this.newNullifiers.slice(i * nullifiersPerBase, (i + 1) * nullifiersPerBase).map(x => x.toBuffer()),
      );
      const publicDataUpdateRequestsBuffer = Buffer.concat(
        this.newPublicDataWrites
          .slice(i * publicDataUpdateRequestsPerBase, (i + 1) * publicDataUpdateRequestsPerBase)
          .map(x => x.toBuffer()),
      );
      const newL2ToL1MsgsBuffer = Buffer.concat(
        this.newL2ToL1Msgs.slice(i * l2ToL1MsgsPerBase, (i + 1) * l2ToL1MsgsPerBase).map(x => x.toBuffer()),
      );
      const encryptedLogsHashKernel0 = L2Block.computeKernelLogsHash(this.newEncryptedLogs.txLogs[i * 2]);
      const encryptedLogsHashKernel1 = L2Block.computeKernelLogsHash(this.newEncryptedLogs.txLogs[i * 2 + 1]);

      const unencryptedLogsHashKernel0 = L2Block.computeKernelLogsHash(this.newUnencryptedLogs.txLogs[i * 2]);
      const unencryptedLogsHashKernel1 = L2Block.computeKernelLogsHash(this.newUnencryptedLogs.txLogs[i * 2 + 1]);

      const inputValue = Buffer.concat([
        commitmentsBuffer,
        nullifiersBuffer,
        publicDataUpdateRequestsBuffer,
        newL2ToL1MsgsBuffer,
        this.newContracts[i * 2].toBuffer(),
        this.newContracts[i * 2 + 1].toBuffer(),
        this.newContractData[i * 2].contractAddress.toBuffer(),
        this.newContractData[i * 2].portalContractAddress.toBuffer32(),
        this.newContractData[i * 2 + 1].contractAddress.toBuffer(),
        this.newContractData[i * 2 + 1].portalContractAddress.toBuffer32(),
        encryptedLogsHashKernel0,
        encryptedLogsHashKernel1,
        unencryptedLogsHashKernel0,
        unencryptedLogsHashKernel1,
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
    if (txIndex >= this.numberOfTxs) {
      throw new Error(
        `Failed to get tx ${txIndex}. Block ${this.globalVariables.blockNumber} only has ${this.numberOfTxs} txs.`,
      );
    }

    const newCommitments = this.newCommitments
      .slice(MAX_NEW_COMMITMENTS_PER_TX * txIndex, MAX_NEW_COMMITMENTS_PER_TX * (txIndex + 1))
      .filter(x => !x.isZero());
    const newNullifiers = this.newNullifiers
      .slice(MAX_NEW_NULLIFIERS_PER_TX * txIndex, MAX_NEW_NULLIFIERS_PER_TX * (txIndex + 1))
      .filter(x => !x.isZero());
    const newPublicDataWrites = this.newPublicDataWrites
      .slice(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * txIndex, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * (txIndex + 1))
      .filter(x => !x.isEmpty());
    const newL2ToL1Msgs = this.newL2ToL1Msgs
      .slice(MAX_NEW_L2_TO_L1_MSGS_PER_TX * txIndex, MAX_NEW_L2_TO_L1_MSGS_PER_TX * (txIndex + 1))
      .filter(x => !x.isZero());
    const newContracts = this.newContracts
      .slice(MAX_NEW_CONTRACTS_PER_TX * txIndex, MAX_NEW_CONTRACTS_PER_TX * (txIndex + 1))
      .filter(x => !x.isZero());
    const newContractData = this.newContractData
      .slice(MAX_NEW_CONTRACTS_PER_TX * txIndex, MAX_NEW_CONTRACTS_PER_TX * (txIndex + 1))
      .filter(x => !x.isEmpty());

    return new L2Tx(
      newCommitments,
      newNullifiers,
      newPublicDataWrites,
      newL2ToL1Msgs,
      newContracts,
      newContractData,
      this.getBlockHash(),
      this.number,
    );
  }

  /**
   * Get all the transaction in an L2 block.
   * @returns The tx.
   */
  getTxs() {
    return Array(this.numberOfTxs)
      .fill(0)
      .map((_, i) => this.getTx(i));
  }

  /**
   * Returns stats used for logging.
   * @returns Stats on tx count, number, and log size and count.
   */
  getStats() {
    const encryptedLogsStats = this.newEncryptedLogs && {
      encryptedLogCount: this.newEncryptedLogs?.getTotalLogCount() ?? 0,
      encryptedLogSize: this.newEncryptedLogs?.getSerializedLength() ?? 0,
    };
    const unencryptedLogsStats = this.newUnencryptedLogs && {
      unencryptedLogCount: this.newUnencryptedLogs?.getTotalLogCount() ?? 0,
      unencryptedLogSize: this.newUnencryptedLogs?.getSerializedLength() ?? 0,
    };
    return {
      txCount: this.numberOfTxs,
      blockNumber: this.number,
      ...encryptedLogsStats,
      ...unencryptedLogsStats,
    };
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
    const inspectGlobalVariables = (gv: GlobalVariables): string => {
      return `(${gv.chainId}, ${gv.version}, ${gv.blockNumber}, ${gv.timestamp}))`;
    };
    const inspectFrArray = (arr: Fr[]): string => inspectArray(arr, inspectHex);
    const inspectContractDataArray = (arr: ContractData[]): string =>
      inspectArray(arr, cd => `(${inspectHex(cd.contractAddress)}, ${inspectHex(cd.portalContractAddress)})`);
    const inspectPublicDataWriteArray = (arr: PublicDataWrite[]): string =>
      inspectArray(arr, pdw => `(${inspectHex(pdw.leafIndex)}, ${inspectHex(pdw.newValue)})`);

    return [
      `L2Block`,
      `number: ${this.number}`,
      `globalVariables: ${inspectGlobalVariables(this.globalVariables)}`,
      `startNoteHashTreeSnapshot: ${inspectTreeSnapshot(this.startNoteHashTreeSnapshot)}`,
      `startNullifierTreeSnapshot: ${inspectTreeSnapshot(this.startNullifierTreeSnapshot)}`,
      `startContractTreeSnapshot: ${inspectTreeSnapshot(this.startContractTreeSnapshot)}`,
      `startPublicDataTreeSnapshot: ${this.startPublicDataTreeSnapshot.toString()}`,
      `startL1ToL2MessagesTreeSnapshot: ${inspectTreeSnapshot(this.startL1ToL2MessagesTreeSnapshot)}`,
      `startArchiveSnapshot: ${inspectTreeSnapshot(this.startArchiveSnapshot)}`,
      `endNoteHashTreeSnapshot: ${inspectTreeSnapshot(this.endNoteHashTreeSnapshot)}`,
      `endNullifierTreeSnapshot: ${inspectTreeSnapshot(this.endNullifierTreeSnapshot)}`,
      `endContractTreeSnapshot: ${inspectTreeSnapshot(this.endContractTreeSnapshot)}`,
      `endPublicDataTreeSnapshot: ${this.endPublicDataTreeSnapshot.toString()}`,
      `endPublicDataTreeSnapshot: ${this.endPublicDataTreeSnapshot.toString()}`,
      `endL1ToL2MessagesTreeSnapshot: ${inspectTreeSnapshot(this.endL1ToL2MessagesTreeSnapshot)}`,
      `endArchiveSnapshot: ${inspectTreeSnapshot(this.endArchiveSnapshot)}`,
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

  /**
   * Computes logs hash as is done in the kernel and app circuits.
   * @param logs - Logs to be hashed.
   * @returns The hash of the logs.
   * Note: This is a TS implementation of `computeKernelLogsHash` function in Decoder.sol. See that function documentation
   *       for more details.
   */
  static computeKernelLogsHash(logs: TxL2Logs): Buffer {
    const logsHashes: [Buffer, Buffer] = [Buffer.alloc(32), Buffer.alloc(32)];
    let kernelPublicInputsLogsHash = Buffer.alloc(32);

    for (const functionLogs of logs.functionLogs) {
      logsHashes[0] = kernelPublicInputsLogsHash;
      logsHashes[1] = functionLogs.hash(); // privateCircuitPublicInputsLogsHash

      // Hash logs hash from the public inputs of previous kernel iteration and logs hash from private circuit public inputs
      kernelPublicInputsLogsHash = sha256(Buffer.concat(logsHashes));
    }

    return kernelPublicInputsLogsHash;
  }
}
