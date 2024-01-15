import {
  AppendOnlyTreeSnapshot,
  GlobalVariables,
  Header,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BYTES_PER_SHA256,
  PartialStateReference,
  STRING_ENCODING,
  StateReference,
} from '@aztec/circuits.js';
import { makeAppendOnlyTreeSnapshot, makeGlobalVariables, makeHeader } from '@aztec/circuits.js/factories';
import { keccak, sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import times from 'lodash.times';

import { ContractData } from './contract_data.js';
import { L2Tx } from './l2_tx.js';
import { LogType, TxL2Logs } from './logs/index.js';
import { L2BlockL2Logs } from './logs/l2_block_l2_logs.js';
import { PublicDataWrite } from './public_data_write.js';
import { TxHash } from './tx/tx_hash.js';

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
    /** Snapshot of archive tree after the block is applied. */
    public archive: AppendOnlyTreeSnapshot,
    /** L2 block header. */
    public header: Header,
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

  get number(): number {
    return Number(this.header.globalVariables.blockNumber.toBigInt());
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
    const globalVariables = makeGlobalVariables(0, l2BlockNum);

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
        archive: makeAppendOnlyTreeSnapshot(1),
        header: makeHeader(0, globalVariables),
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
      /** Snapshot of archive tree after the block is applied. */
      archive: AppendOnlyTreeSnapshot;
      /** L2 block header. */
      header: Header;
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
       * The L1 to L2 messages to be inserted into the L1 to L2 message tree.
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
      fields.archive,
      fields.header,
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
      this.header.globalVariables,
      // TODO(#3868)
      AppendOnlyTreeSnapshot.empty(), // this.startNoteHashTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startNullifierTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startContractTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startPublicDataTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startL1ToL2MessageTreeSnapshot,
      this.header.lastArchive,
      this.header.state.partial.noteHashTree,
      this.header.state.partial.nullifierTree,
      this.header.state.partial.contractTree,
      this.header.state.partial.publicDataTree,
      this.header.state.l1ToL2MessageTree,
      this.archive,
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
        `newEncryptedLogs and newUnencryptedLogs must be defined when encoding L2BlockData (block ${this.header.globalVariables.blockNumber})`,
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
   * @param blockHash - The hash of the block.
   * @returns Deserialized L2 block.
   */
  static fromBuffer(buf: Buffer | BufferReader, blockHash?: Buffer) {
    const reader = BufferReader.asReader(buf);
    const globalVariables = reader.readObject(GlobalVariables);
    // TODO(#3938): update the encoding here
    reader.readObject(AppendOnlyTreeSnapshot); // startNoteHashTreeSnapshot
    reader.readObject(AppendOnlyTreeSnapshot); // startNullifierTreeSnapshot
    reader.readObject(AppendOnlyTreeSnapshot); // startContractTreeSnapshot
    reader.readObject(AppendOnlyTreeSnapshot); // startPublicDataTreeSnapshot
    reader.readObject(AppendOnlyTreeSnapshot); // startL1ToL2MessageTreeSnapshot
    const startArchiveSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endNoteHashTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endPublicDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endL1ToL2MessageTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endArchiveSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const newCommitments = reader.readVector(Fr);
    const newNullifiers = reader.readVector(Fr);
    const newPublicDataWrites = reader.readVector(PublicDataWrite);
    const newL2ToL1Msgs = reader.readVector(Fr);
    const newContracts = reader.readVector(Fr);
    const newContractData = reader.readArray(newContracts.length, ContractData);
    // TODO(sean): could an optimization of this be that it is encoded such that zeros are assumed
    const newL1ToL2Messages = reader.readVector(Fr);

    const partial = new PartialStateReference(
      endNoteHashTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endPublicDataTreeSnapshot,
    );
    const state = new StateReference(endL1ToL2MessageTreeSnapshot, partial);
    // TODO(#3938): populate bodyHash
    const header = new Header(startArchiveSnapshot, Buffer.alloc(NUM_BYTES_PER_SHA256), state, globalVariables);

    return L2Block.fromFields(
      {
        archive: endArchiveSnapshot,
        header,
        newCommitments,
        newNullifiers,
        newPublicDataWrites,
        newL2ToL1Msgs,
        newContracts,
        newContractData,
        newL1ToL2Messages,
      },
      blockHash,
    );
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
      throw new Error(
        `Trying to attach different ${logFieldName} logs to block ${this.header.globalVariables.blockNumber}.`,
      );
    }

    L2Block.logger(
      `Attaching ${logFieldName} ${logs.getTotalLogCount()} logs to block ${this.header.globalVariables.blockNumber}`,
    );

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
      this.header.globalVariables,
      // TODO(#3868)
      AppendOnlyTreeSnapshot.empty(), // this.startNoteHashTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startNullifierTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startContractTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startPublicDataTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startL1ToL2MessageTreeSnapshot,
      this.header.lastArchive,
      this.header.state.partial.noteHashTree,
      this.header.state.partial.nullifierTree,
      this.header.state.partial.contractTree,
      this.header.state.partial.publicDataTree,
      this.header.state.l1ToL2MessageTree,
      this.archive,
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
      new Fr(Number(this.header.globalVariables.blockNumber.toBigInt()) - 1),
      // TODO(#3868)
      AppendOnlyTreeSnapshot.empty(), // this.startNoteHashTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startNullifierTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startContractTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startPublicDataTreeSnapshot,
      AppendOnlyTreeSnapshot.empty(), // this.startL1ToL2MessageTreeSnapshot,
      this.header.lastArchive,
    );
    return sha256(inputValue);
  }

  /**
   * Computes the end state hash (should equal contract data after block).
   * @returns The end state hash for the L2 block.
   */
  getEndStateHash() {
    const inputValue = serializeToBuffer(
      this.header.globalVariables.blockNumber,
      this.header.state.partial.noteHashTree,
      this.header.state.partial.nullifierTree,
      this.header.state.partial.contractTree,
      this.header.state.partial.publicDataTree,
      this.header.state.l1ToL2MessageTree,
      this.archive,
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

    const leafCount = this.newCommitments.length / MAX_NEW_COMMITMENTS_PER_TX;
    const leafs: Buffer[] = [];

    for (let i = 0; i < leafCount; i++) {
      const commitmentsPerBase = MAX_NEW_COMMITMENTS_PER_TX;
      const nullifiersPerBase = MAX_NEW_NULLIFIERS_PER_TX;
      const publicDataUpdateRequestsPerBase = MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX;
      const l2ToL1MsgsPerBase = MAX_NEW_L2_TO_L1_MSGS_PER_TX;
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
      const encryptedLogsHashKernel0 = L2Block.computeKernelLogsHash(this.newEncryptedLogs.txLogs[i]);

      const unencryptedLogsHashKernel0 = L2Block.computeKernelLogsHash(this.newUnencryptedLogs.txLogs[i]);

      const inputValue = Buffer.concat([
        commitmentsBuffer,
        nullifiersBuffer,
        publicDataUpdateRequestsBuffer,
        newL2ToL1MsgsBuffer,
        this.newContracts[i].toBuffer(),
        this.newContractData[i].contractAddress.toBuffer(),
        this.newContractData[i].portalContractAddress.toBuffer32(),
        encryptedLogsHashKernel0,
        unencryptedLogsHashKernel0,
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
    this.assertIndexInRange(txIndex);

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
      Number(this.header.globalVariables.blockNumber.toBigInt()),
    );
  }

  /**
   * A lightweight method to get the tx hash of a tx in the block.
   * @param txIndex - the index of the tx in the block
   * @returns a hash of the tx, which is the first nullifier in the tx
   */
  getTxHash(txIndex: number): TxHash {
    this.assertIndexInRange(txIndex);

    const firstNullifier = this.newNullifiers[txIndex * MAX_NEW_NULLIFIERS_PER_TX];

    return new TxHash(firstNullifier.toBuffer());
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

  assertIndexInRange(txIndex: number) {
    if (txIndex < 0 || txIndex >= this.numberOfTxs) {
      throw new IndexOutOfRangeError({
        txIndex,
        numberOfTxs: this.numberOfTxs,
        blockNumber: this.number,
      });
    }
  }

  // /**
  //  * Inspect for debugging purposes..
  //  * @param maxBufferSize - The number of bytes to be extracted from buffer.
  //  * @returns A human-friendly string representation of the l2Block.
  //  */
  // inspect(maxBufferSize = 4): string {
  //   const inspectHex = (fr: {
  //     /**
  //      * A function used to serialize the field element to a buffer.
  //      */
  //     toBuffer: () => Buffer;
  //   }): string => `0x${fr.toBuffer().subarray(0, maxBufferSize).toString('hex')}`;
  //   const inspectArray = <T>(arr: T[], inspector: (t: T) => string) => '[' + arr.map(inspector).join(', ') + ']';

  //   const inspectTreeSnapshot = (s: AppendOnlyTreeSnapshot): string =>
  //     `(${s.nextAvailableLeafIndex}, ${inspectHex(s.root)})`;
  //   const inspectGlobalVariables = (gv: GlobalVariables): string => {
  //     return `(${gv.chainId}, ${gv.version}, ${gv.blockNumber}, ${gv.timestamp}))`;
  //   };
  //   const inspectFrArray = (arr: Fr[]): string => inspectArray(arr, inspectHex);
  //   const inspectContractDataArray = (arr: ContractData[]): string =>
  //     inspectArray(arr, cd => `(${inspectHex(cd.contractAddress)}, ${inspectHex(cd.portalContractAddress)})`);
  //   const inspectPublicDataWriteArray = (arr: PublicDataWrite[]): string =>
  //     inspectArray(arr, pdw => `(${inspectHex(pdw.leafIndex)}, ${inspectHex(pdw.newValue)})`);

  //   return [
  //     `L2Block`,
  //     `number: ${this.header.globalVariables.blockNumber}`,
  //     `globalVariables: ${inspectGlobalVariables(this.globalVariables)}`,
  //     `startNoteHashTreeSnapshot: ${inspectTreeSnapshot(this.startNoteHashTreeSnapshot)}`,
  //     `startNullifierTreeSnapshot: ${inspectTreeSnapshot(this.startNullifierTreeSnapshot)}`,
  //     `startContractTreeSnapshot: ${inspectTreeSnapshot(this.startContractTreeSnapshot)}`,
  //     `startPublicDataTreeSnapshot: ${this.startPublicDataTreeSnapshot.toString()}`,
  //     `startL1ToL2MessageTreeSnapshot: ${inspectTreeSnapshot(this.startL1ToL2MessageTreeSnapshot)}`,
  //     `startArchiveSnapshot: ${inspectTreeSnapshot(this.startArchiveSnapshot)}`,
  //     `endNoteHashTreeSnapshot: ${inspectTreeSnapshot(this.endNoteHashTreeSnapshot)}`,
  //     `endNullifierTreeSnapshot: ${inspectTreeSnapshot(this.endNullifierTreeSnapshot)}`,
  //     `endContractTreeSnapshot: ${inspectTreeSnapshot(this.endContractTreeSnapshot)}`,
  //     `endPublicDataTreeSnapshot: ${this.endPublicDataTreeSnapshot.toString()}`,
  //     `endPublicDataTreeSnapshot: ${this.endPublicDataTreeSnapshot.toString()}`,
  //     `endL1ToL2MessageTreeSnapshot: ${inspectTreeSnapshot(this.endL1ToL2MessageTreeSnapshot)}`,
  //     `endArchiveSnapshot: ${inspectTreeSnapshot(this.endArchiveSnapshot)}`,
  //     `newCommitments: ${inspectFrArray(this.newCommitments)}`,
  //     `newNullifiers: ${inspectFrArray(this.newNullifiers)}`,
  //     `newPublicDataWrite: ${inspectPublicDataWriteArray(this.newPublicDataWrites)}`,
  //     `newL2ToL1Msgs: ${inspectFrArray(this.newL2ToL1Msgs)}`,
  //     `newContracts: ${inspectFrArray(this.newContracts)}`,
  //     `newContractData: ${inspectContractDataArray(this.newContractData)}`,
  //     `newPublicDataWrite: ${inspectPublicDataWriteArray(this.newPublicDataWrites)}`,
  //     `newL1ToL2Messages: ${inspectFrArray(this.newL1ToL2Messages)}`,
  //   ].join('\n');
  // }

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

/**
 * Custom error class for when a requested tx index is out of range.
 */
export class IndexOutOfRangeError extends Error {
  constructor({
    txIndex,
    numberOfTxs,
    blockNumber,
  }: {
    /**
     * The requested index of the tx in the block.
     */
    txIndex: number;
    /**
     * The number of txs in the block.
     */
    numberOfTxs: number;
    /**
     * The number of the block.
     */
    blockNumber: number;
  }) {
    super(`IndexOutOfRangeError: Failed to get tx at index ${txIndex}. Block ${blockNumber} has ${numberOfTxs} txs.`);
  }
}
