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
import { keccak, sha256, sha256ToField } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import times from 'lodash.times';

import { ContractData, L2Tx, LogType, PublicDataWrite, TxL2Logs } from './index.js';
import { L2BlockL2Logs } from './logs/l2_block_l2_logs.js';

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
     * The tree root of the public data tree at the start of the rollup.
     */
    public startPublicDataTreeRoot: Fr,
    /**
     * The tree snapshot of the L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic blocks tree at the start of the rollup.
     */
    public startHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot = AppendOnlyTreeSnapshot.empty(),
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
     * The tree root of the public data tree at the end of the rollup.
     */
    public endPublicDataTreeRoot: Fr,
    /**
     * The tree snapshot of the L2 message tree at the end of the rollup.
     */
    public endL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * The tree snapshot of the historic blocks tree at the end of the rollup.
     */
    public endHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot,
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
    newEncryptedLogs?: L2BlockL2Logs,
    newUnencryptedLogs?: L2BlockL2Logs,
    private blockHash?: Buffer,
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

    this.numberOfTxs = Math.floor(this.newCommitments.length / MAX_NEW_COMMITMENTS_PER_TX);
  }

  /**
   * Creates an L2 block containing random data.
   * @param l2BlockNum - The number of the L2 block.
   * @param txsPerBlock - The number of transactions to include in the block.
   * @param numPrivateFunctionCalls - The number of private function calls to include in each transaction.
   * @param numPublicFunctionCalls - The number of public function calls to include in each transaction.
   * @param numEncryptedLogs - The number of encrypted logs to include in each transaction.
   * @param numUnencryptedLogs - The number of unencrypted logs to include in each transaction.
   * @returns The L2 block.
   */
  static random(
    l2BlockNum: number,
    txsPerBlock = 4,
    numPrivateFunctionCalls = 2,
    numPublicFunctionCalls = 3,
    numEncryptedLogs = 2,
    numUnencryptedLogs = 1,
  ): L2Block {
    const newNullifiers = times(MAX_NEW_NULLIFIERS_PER_TX * txsPerBlock, Fr.random);
    const newCommitments = times(MAX_NEW_COMMITMENTS_PER_TX * txsPerBlock, Fr.random);
    const newContracts = times(MAX_NEW_CONTRACTS_PER_TX * txsPerBlock, Fr.random);
    const newContractData = times(MAX_NEW_CONTRACTS_PER_TX * txsPerBlock, ContractData.random);
    const newPublicDataWrites = times(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * txsPerBlock, PublicDataWrite.random);
    const newL1ToL2Messages = times(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, Fr.random);
    const newL2ToL1Msgs = times(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.random);
    const newEncryptedLogs = L2BlockL2Logs.random(txsPerBlock, numPrivateFunctionCalls, numEncryptedLogs);
    const newUnencryptedLogs = L2BlockL2Logs.random(txsPerBlock, numPublicFunctionCalls, numUnencryptedLogs);

    return L2Block.fromFields({
      number: l2BlockNum,
      globalVariables: makeGlobalVariables(0, l2BlockNum),
      startPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startContractTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startPublicDataTreeRoot: Fr.random(),
      startL1ToL2MessagesTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      startHistoricBlocksTreeSnapshot: makeAppendOnlyTreeSnapshot(0),
      endPrivateDataTreeSnapshot: makeAppendOnlyTreeSnapshot(newCommitments.length),
      endNullifierTreeSnapshot: makeAppendOnlyTreeSnapshot(newNullifiers.length),
      endContractTreeSnapshot: makeAppendOnlyTreeSnapshot(newContracts.length),
      endPublicDataTreeRoot: Fr.random(),
      endL1ToL2MessagesTreeSnapshot: makeAppendOnlyTreeSnapshot(1),
      endHistoricBlocksTreeSnapshot: makeAppendOnlyTreeSnapshot(1),
      newCommitments,
      newNullifiers,
      newContracts,
      newContractData,
      newPublicDataWrites,
      newL1ToL2Messages,
      newL2ToL1Msgs,
      newEncryptedLogs,
      newUnencryptedLogs,
    });
  }

  /**
   * Constructs a new instance from named fields.
   * @param fields - Fields to pass to the constructor.
   * @param blockHash - Hash of the block.
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
       * The tree root of the public data tree at the start of the rollup.
       */
      startPublicDataTreeRoot: Fr;
      /**
       * The tree snapshot of the L2 message tree at the start of the rollup.
       */
      startL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the historic blocks tree at the start of the rollup.
       */
      startHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot;
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
       * The tree root of the public data tree at the end of the rollup.
       */
      endPublicDataTreeRoot: Fr;
      /**
       * The tree snapshot of the L2 message tree at the end of the rollup.
       */
      endL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot;
      /**
       * The tree snapshot of the historic blocks tree at the end of the rollup.
       */
      endHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot;
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
  ) {
    return new this(
      fields.number,
      fields.globalVariables,
      fields.startPrivateDataTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.startPublicDataTreeRoot,
      fields.startL1ToL2MessagesTreeSnapshot,
      fields.startHistoricBlocksTreeSnapshot,
      fields.endPrivateDataTreeSnapshot,
      fields.endNullifierTreeSnapshot,
      fields.endContractTreeSnapshot,
      fields.endPublicDataTreeRoot,
      fields.endL1ToL2MessagesTreeSnapshot,
      fields.endHistoricBlocksTreeSnapshot,
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
    );
  }

  /**
   * Encode the L2 block data into a buffer that can be pushed to the rollup contract.
   * @returns The encoded L2 block data.
   */
  encode(): Buffer {
    if (this.newEncryptedLogs === undefined || this.newUnencryptedLogs === undefined) {
      throw new Error('newEncryptedLogs and newUnencryptedLogs must be defined when encoding L2BlockData');
    }

    return serializeToBuffer(
      this.globalVariables,
      this.startPrivateDataTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startPublicDataTreeRoot,
      this.startL1ToL2MessagesTreeSnapshot,
      this.startHistoricBlocksTreeSnapshot,
      this.endPrivateDataTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endPublicDataTreeRoot,
      this.endL1ToL2MessagesTreeSnapshot,
      this.endHistoricBlocksTreeSnapshot,
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
      this.newEncryptedLogs,
      this.newUnencryptedLogs,
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
   * Encodes the block as a hex string
   * @returns The encoded L2 block data as a hex string.
   */
  toString() {
    return this.toBuffer().toString(STRING_ENCODING);
  }

  /**
   * Encodes the block as a JSON object.
   * @returns The L2 block encoded as a JSON object.
   */
  toJSON() {
    return {
      globalVariables: this.globalVariables.toJSON(),
      startPrivateDataTreeSnapshot: this.startPrivateDataTreeSnapshot.toString(),
      startNullifierTreeSnapshot: this.startNullifierTreeSnapshot.toString(),
      startContractTreeSnapshot: this.startContractTreeSnapshot.toString(),
      startPublicDataTreeRoot: this.startPublicDataTreeRoot.toString(),
      startL1ToL2MessagesTreeSnapshot: this.startL1ToL2MessagesTreeSnapshot.toString(),
      startHistoricBlocksTreeSnapshot: this.startHistoricBlocksTreeSnapshot.toString(),
      endPrivateDataTreeSnapshot: this.endPrivateDataTreeSnapshot.toString(),
      endNullifierTreeSnapshot: this.endNullifierTreeSnapshot.toString(),
      endContractTreeSnapshot: this.endContractTreeSnapshot.toString(),
      endPublicDataTreeRoot: this.endPublicDataTreeRoot.toString(),
      endL1ToL2MessagesTreeSnapshot: this.endL1ToL2MessagesTreeSnapshot.toString(),
      endHistoricBlocksTreeSnapshot: this.endHistoricBlocksTreeSnapshot.toString(),
      newCommitments: this.newCommitments.map(c => c.toString()),
      newNullifiers: this.newNullifiers.map(n => n.toString()),
      newPublicDataWrites: this.newPublicDataWrites.map(p => p.toString()),
      newL2ToL1Msgs: this.newL2ToL1Msgs.map(m => m.toString()),
      newContracts: this.newContracts.map(c => c.toString()),
      newContractData: this.newContractData.map(c => c.toString()),
      newL1ToL2Messages: this.newL1ToL2Messages.map(m => m.toString()),
      newEncryptedLogs: this.newEncryptedLogs?.toJSON() ?? null,
      newUnencryptedLogs: this.newUnencryptedLogs?.toJSON() ?? null,
    };
  }

  /**
   * Decode the L2 block data from a buffer.
   * @param encoded - The encoded L2 block data.
   * @returns The decoded L2 block data.
   */
  static decode(encoded: Buffer | BufferReader) {
    const reader = BufferReader.asReader(encoded);
    const globalVariables = reader.readObject(GlobalVariables);
    const number = Number(globalVariables.blockNumber.value);
    const startPrivateDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startPublicDataTreeRoot = reader.readObject(Fr);
    const startL1ToL2MessagesTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const startHistoricBlocksTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endPrivateDataTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endNullifierTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endContractTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endPublicDataTreeRoot = reader.readObject(Fr);
    const endL1ToL2MessagesTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const endHistoricBlocksTreeSnapshot = reader.readObject(AppendOnlyTreeSnapshot);
    const newCommitments = reader.readVector(Fr);
    const newNullifiers = reader.readVector(Fr);
    const newPublicDataWrites = reader.readVector(PublicDataWrite);
    const newL2ToL1Msgs = reader.readVector(Fr);
    const newContracts = reader.readVector(Fr);
    const newContractData = reader.readArray(newContracts.length, ContractData);
    // TODO(sean): could an optimisation of this be that it is encoded such that zeros are assumed
    const newL1ToL2Messages = reader.readVector(Fr);
    const newEncryptedLogs = reader.readObject(L2BlockL2Logs);
    const newUnencryptedLogs = reader.readObject(L2BlockL2Logs);

    return L2Block.fromFields({
      number,
      globalVariables,
      startPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startPublicDataTreeRoot,
      startL1ToL2MessagesTreeSnapshot: startL1ToL2MessagesTreeSnapshot,
      startHistoricBlocksTreeSnapshot,
      endPrivateDataTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endPublicDataTreeRoot,
      endL1ToL2MessagesTreeSnapshot,
      endHistoricBlocksTreeSnapshot,
      newCommitments,
      newNullifiers,
      newPublicDataWrites,
      newL2ToL1Msgs,
      newContracts,
      newContractData,
      newL1ToL2Messages,
      newEncryptedLogs,
      newUnencryptedLogs,
    });
  }

  /**
   * Decode the L2 block from a string
   * @param str - The serialised L2 block
   * @returns An L2 block
   */
  static fromString(str: string): L2Block {
    return L2Block.decode(Buffer.from(str, STRING_ENCODING));
  }

  static fromJSON(_obj: any): L2Block {
    const globalVariables = GlobalVariables.fromJSON(_obj.globalVariables);
    const number = Number(globalVariables.blockNumber.value);
    const startPrivateDataTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.startPrivateDataTreeSnapshot);
    const startNullifierTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.startNullifierTreeSnapshot);
    const startContractTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.startContractTreeSnapshot);
    const startPublicDataTreeRoot = Fr.fromString(_obj.startPublicDataTreeRoot);
    const startL1ToL2MessagesTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.startL1ToL2MessagesTreeSnapshot);
    const startHistoricBlocksTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.startHistoricBlocksTreeSnapshot);
    const endPrivateDataTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.endPrivateDataTreeSnapshot);
    const endNullifierTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.endNullifierTreeSnapshot);
    const endContractTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.endContractTreeSnapshot);
    const endPublicDataTreeRoot = Fr.fromString(_obj.endPublicDataTreeRoot);
    const endL1ToL2MessagesTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.endL1ToL2MessagesTreeSnapshot);
    const endHistoricBlocksTreeSnapshot = AppendOnlyTreeSnapshot.fromString(_obj.endHistoricBlocksTreeSnapshot);
    const newCommitments = _obj.newCommitments.map((c: string) => Fr.fromString(c));
    const newNullifiers = _obj.newNullifiers.map((n: string) => Fr.fromString(n));
    const newPublicDataWrites = _obj.newPublicDataWrites.map((p: any) => PublicDataWrite.fromString(p));
    const newL2ToL1Msgs = _obj.newL2ToL1Msgs.map((m: string) => Fr.fromString(m));
    const newContracts = _obj.newContracts.map((c: string) => Fr.fromString(c));
    const newContractData = _obj.newContractData.map((c: any) => ContractData.fromString(c));
    const newL1ToL2Messages = _obj.newL1ToL2Messages.map((m: string) => Fr.fromString(m));
    const newEncryptedLogs = _obj.newEncryptedLogs ? L2BlockL2Logs.fromJSON(_obj.newEncryptedLogs) : undefined;
    const newUnencryptedLogs = _obj.newUnencryptedLogs ? L2BlockL2Logs.fromJSON(_obj.newUnencryptedLogs) : undefined;

    return L2Block.fromFields({
      number,
      globalVariables,
      startPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startPublicDataTreeRoot,
      startL1ToL2MessagesTreeSnapshot,
      startHistoricBlocksTreeSnapshot,
      endPrivateDataTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endPublicDataTreeRoot,
      endL1ToL2MessagesTreeSnapshot,
      endHistoricBlocksTreeSnapshot,
      newCommitments,
      newNullifiers,
      newPublicDataWrites,
      newL2ToL1Msgs,
      newContracts,
      newContractData,
      newL1ToL2Messages,
      newEncryptedLogs,
      newUnencryptedLogs,
    });
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

    L2Block.logger(`Attaching ${logFieldName} logs`);

    const numTxs = this.newCommitments.length / MAX_NEW_COMMITMENTS_PER_TX;

    if (numTxs !== logs.txLogs.length) {
      throw new Error(
        `Number of txLogs within ${logFieldName} does not match number of transactions. Expected: ${numTxs} Got: ${logs.txLogs.length}`,
      );
    }

    this[logFieldName] = logs;
  }

  /**
   * Returns the block's hash.
   * @returns The block's hash.
   */
  public getBlockHash(): Buffer {
    if (!this.blockHash) {
      this.blockHash = keccak(this.encode());
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
      this.startPrivateDataTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startPublicDataTreeRoot,
      this.startL1ToL2MessagesTreeSnapshot,
      this.startHistoricBlocksTreeSnapshot,
      this.endPrivateDataTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endPublicDataTreeRoot,
      this.endL1ToL2MessagesTreeSnapshot,
      this.endHistoricBlocksTreeSnapshot,
      this.getCalldataHash(),
      this.getL1ToL2MessagesHash(),
    );

    return sha256ToField(buf);
  }

  /**
   * Computes the start state hash (should equal contract data before block).
   * @returns The start state hash for the L2 block.
   */
  getStartStateHash() {
    const inputValue = serializeToBuffer(
      new Fr(this.number - 1),
      this.startPrivateDataTreeSnapshot,
      this.startNullifierTreeSnapshot,
      this.startContractTreeSnapshot,
      this.startPublicDataTreeRoot,
      this.startL1ToL2MessagesTreeSnapshot,
      this.startHistoricBlocksTreeSnapshot,
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
      this.endPrivateDataTreeSnapshot,
      this.endNullifierTreeSnapshot,
      this.endContractTreeSnapshot,
      this.endPublicDataTreeRoot,
      this.endL1ToL2MessagesTreeSnapshot,
      this.endHistoricBlocksTreeSnapshot,
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

    const newCommitments = this.newCommitments.slice(
      MAX_NEW_COMMITMENTS_PER_TX * txIndex,
      MAX_NEW_COMMITMENTS_PER_TX * (txIndex + 1),
    );
    const newNullifiers = this.newNullifiers.slice(
      MAX_NEW_NULLIFIERS_PER_TX * txIndex,
      MAX_NEW_NULLIFIERS_PER_TX * (txIndex + 1),
    );
    const newPublicDataWrites = this.newPublicDataWrites.slice(
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * txIndex,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * (txIndex + 1),
    );
    const newL2ToL1Msgs = this.newL2ToL1Msgs.slice(
      MAX_NEW_L2_TO_L1_MSGS_PER_TX * txIndex,
      MAX_NEW_L2_TO_L1_MSGS_PER_TX * (txIndex + 1),
    );
    const newContracts = this.newContracts.slice(
      MAX_NEW_CONTRACTS_PER_TX * txIndex,
      MAX_NEW_CONTRACTS_PER_TX * (txIndex + 1),
    );
    const newContractData = this.newContractData.slice(
      MAX_NEW_CONTRACTS_PER_TX * txIndex,
      MAX_NEW_CONTRACTS_PER_TX * (txIndex + 1),
    );

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
   * @returns The txx.
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
    return {
      txCount: this.numberOfTxs,
      blockNumber: this.number,
      encryptedLogCount: this.newEncryptedLogs?.getTotalLogCount() ?? 0,
      unencryptedLogCount: this.newUnencryptedLogs?.getTotalLogCount() ?? 0,
      encryptedLogSize: this.newEncryptedLogs?.getSerializedLength() ?? 0,
      unencryptedLogSize: this.newUnencryptedLogs?.getSerializedLength() ?? 0,
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
      `startPrivateDataTreeSnapshot: ${inspectTreeSnapshot(this.startPrivateDataTreeSnapshot)}`,
      `startNullifierTreeSnapshot: ${inspectTreeSnapshot(this.startNullifierTreeSnapshot)}`,
      `startContractTreeSnapshot: ${inspectTreeSnapshot(this.startContractTreeSnapshot)}`,
      `startPublicDataTreeRoot: ${this.startPublicDataTreeRoot.toString()}`,
      `startL1ToL2MessagesTreeSnapshot: ${inspectTreeSnapshot(this.startL1ToL2MessagesTreeSnapshot)}`,
      `startHistoricBlocksTreeSnapshot: ${inspectTreeSnapshot(this.startHistoricBlocksTreeSnapshot)}`,
      `endPrivateDataTreeSnapshot: ${inspectTreeSnapshot(this.endPrivateDataTreeSnapshot)}`,
      `endNullifierTreeSnapshot: ${inspectTreeSnapshot(this.endNullifierTreeSnapshot)}`,
      `endContractTreeSnapshot: ${inspectTreeSnapshot(this.endContractTreeSnapshot)}`,
      `endPublicDataTreeRoot: ${this.endPublicDataTreeRoot.toString()}`,
      `endPublicDataTreeRoot: ${this.endPublicDataTreeRoot.toString()}`,
      `endL1ToL2MessagesTreeSnapshot: ${inspectTreeSnapshot(this.endL1ToL2MessagesTreeSnapshot)}`,
      `endHistoricBlocksTreeSnapshot: ${inspectTreeSnapshot(this.endHistoricBlocksTreeSnapshot)}`,
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
