import {
  type Body,
  type EncryptedL2BlockL2Logs,
  type EncryptedNoteL2BlockL2Logs,
  ExtendedUnencryptedL2Log,
  type FromLogType,
  type GetUnencryptedLogsResponse,
  type InboxLeaf,
  type L2Block,
  type L2BlockL2Logs,
  type LogFilter,
  LogId,
  LogType,
  type TxEffect,
  type TxHash,
  TxReceipt,
  type UnencryptedL2BlockL2Logs,
} from '@aztec/circuit-types';
import { Fr, INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import {
  type ContractClassPublic,
  type ContractInstanceWithAddress,
  type ExecutablePrivateFunctionWithMembershipProof,
  type UnconstrainedFunctionWithMembershipProof,
} from '@aztec/types/contracts';

import { type ArchiverDataStore, type ArchiverL1SynchPoint } from '../archiver_store.js';
import { type DataRetrieval } from '../data_retrieval.js';
import { L1ToL2MessageStore } from './l1_to_l2_message_store.js';

/**
 * Simple, in-memory implementation of an archiver data store.
 */
export class MemoryArchiverStore implements ArchiverDataStore {
  /**
   * An array containing all the L2 blocks that have been fetched so far.
   */
  private l2Blocks: L2Block[] = [];

  /**
   * A mapping of body hash to body
   */
  private l2BlockBodies: Map<string, Body> = new Map();

  /**
   * An array containing all the the tx effects in the L2 blocks that have been fetched so far.
   */
  private txEffects: TxEffect[] = [];

  /**
   * An array containing all the encrypted logs that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private noteEncryptedLogsPerBlock: EncryptedNoteL2BlockL2Logs[] = [];

  /**
   * An array containing all the encrypted logs that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private encryptedLogsPerBlock: EncryptedL2BlockL2Logs[] = [];

  /**
   * An array containing all the unencrypted logs that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private unencryptedLogsPerBlock: UnencryptedL2BlockL2Logs[] = [];

  /**
   * Contains all L1 to L2 messages.
   */
  private l1ToL2Messages = new L1ToL2MessageStore();

  private contractArtifacts: Map<string, ContractArtifact> = new Map();

  private contractClasses: Map<string, ContractClassPublic> = new Map();

  private privateFunctions: Map<string, ExecutablePrivateFunctionWithMembershipProof[]> = new Map();

  private unconstrainedFunctions: Map<string, UnconstrainedFunctionWithMembershipProof[]> = new Map();

  private contractInstances: Map<string, ContractInstanceWithAddress> = new Map();

  private lastL1BlockNewBlocks: bigint = 0n;
  private lastL1BlockNewMessages: bigint = 0n;

  constructor(
    /** The max number of logs that can be obtained in 1 "getUnencryptedLogs" call. */
    public readonly maxLogs: number,
  ) {}

  public getContractClass(id: Fr): Promise<ContractClassPublic | undefined> {
    const contractClass = this.contractClasses.get(id.toString());
    return Promise.resolve(
      contractClass && {
        ...contractClass,
        privateFunctions: this.privateFunctions.get(id.toString()) ?? [],
        unconstrainedFunctions: this.unconstrainedFunctions.get(id.toString()) ?? [],
      },
    );
  }

  public getContractClassIds(): Promise<Fr[]> {
    return Promise.resolve(Array.from(this.contractClasses.keys()).map(key => Fr.fromString(key)));
  }

  public getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return Promise.resolve(this.contractInstances.get(address.toString()));
  }

  public addFunctions(
    contractClassId: Fr,
    newPrivateFunctions: ExecutablePrivateFunctionWithMembershipProof[],
    newUnconstrainedFunctions: UnconstrainedFunctionWithMembershipProof[],
  ): Promise<boolean> {
    const privateFunctions = this.privateFunctions.get(contractClassId.toString()) ?? [];
    const unconstrainedFunctions = this.unconstrainedFunctions.get(contractClassId.toString()) ?? [];
    const updatedPrivateFunctions = [
      ...privateFunctions,
      ...newPrivateFunctions.filter(newFn => !privateFunctions.find(f => f.selector.equals(newFn.selector))),
    ];
    const updatedUnconstrainedFunctions = [
      ...unconstrainedFunctions,
      ...newUnconstrainedFunctions.filter(
        newFn => !unconstrainedFunctions.find(f => f.selector.equals(newFn.selector)),
      ),
    ];
    this.privateFunctions.set(contractClassId.toString(), updatedPrivateFunctions);
    this.unconstrainedFunctions.set(contractClassId.toString(), updatedUnconstrainedFunctions);
    return Promise.resolve(true);
  }

  public addContractClasses(data: ContractClassPublic[], _blockNumber: number): Promise<boolean> {
    for (const contractClass of data) {
      this.contractClasses.set(contractClass.id.toString(), contractClass);
    }
    return Promise.resolve(true);
  }

  public addContractInstances(data: ContractInstanceWithAddress[], _blockNumber: number): Promise<boolean> {
    for (const contractInstance of data) {
      this.contractInstances.set(contractInstance.address.toString(), contractInstance);
    }
    return Promise.resolve(true);
  }

  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store and the last processed L1 block.
   * @returns True if the operation is successful.
   */
  public addBlocks(blocks: DataRetrieval<L2Block>): Promise<boolean> {
    this.lastL1BlockNewBlocks = blocks.lastProcessedL1BlockNumber;
    this.l2Blocks.push(...blocks.retrievedData);
    this.txEffects.push(...blocks.retrievedData.flatMap(b => b.body.txEffects));
    return Promise.resolve(true);
  }

  /**
   * Append new block bodies to the store's list.
   * @param blockBodies - The L2 block bodies to be added to the store.
   * @returns True if the operation is successful.
   */
  addBlockBodies(blockBodies: Body[]): Promise<boolean> {
    for (const body of blockBodies) {
      void this.l2BlockBodies.set(body.getTxsEffectsHash().toString('hex'), body);
    }

    return Promise.resolve(true);
  }

  /**
   * Gets block bodies that have the same txHashes as we supply.
   *
   * @param txsEffectsHashes - A list of txsEffectsHashes (body hashes).
   * @returns The requested L2 block bodies
   */
  getBlockBodies(txsEffectsHashes: Buffer[]): Promise<Body[]> {
    const blockBodies = txsEffectsHashes.map(txsEffectsHash => this.l2BlockBodies.get(txsEffectsHash.toString('hex')));

    if (blockBodies.some(bodyBuffer => bodyBuffer === undefined)) {
      throw new Error('Block body is undefined');
    }

    return Promise.resolve(blockBodies as Body[]);
  }

  /**
   * Append new logs to the store's list.
   * @param encryptedLogs - The encrypted logs to be added to the store.
   * @param unencryptedLogs - The unencrypted logs to be added to the store.
   * @param blockNumber - The block for which to add the logs.
   * @returns True if the operation is successful.
   */
  addLogs(
    noteEncryptedLogs: EncryptedNoteL2BlockL2Logs,
    encryptedLogs: EncryptedL2BlockL2Logs,
    unencryptedLogs: UnencryptedL2BlockL2Logs,
    blockNumber: number,
  ): Promise<boolean> {
    if (noteEncryptedLogs) {
      this.noteEncryptedLogsPerBlock[blockNumber - INITIAL_L2_BLOCK_NUM] = noteEncryptedLogs;
    }

    if (encryptedLogs) {
      this.encryptedLogsPerBlock[blockNumber - INITIAL_L2_BLOCK_NUM] = encryptedLogs;
    }

    if (unencryptedLogs) {
      this.unencryptedLogsPerBlock[blockNumber - INITIAL_L2_BLOCK_NUM] = unencryptedLogs;
    }

    return Promise.resolve(true);
  }

  /**
   * Append L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store and the last processed L1 block.
   * @returns True if the operation is successful.
   */
  public addL1ToL2Messages(messages: DataRetrieval<InboxLeaf>): Promise<boolean> {
    if (messages.lastProcessedL1BlockNumber <= this.lastL1BlockNewMessages) {
      return Promise.resolve(false);
    }

    this.lastL1BlockNewMessages = messages.lastProcessedL1BlockNumber;
    for (const message of messages.retrievedData) {
      this.l1ToL2Messages.addMessage(message);
    }
    return Promise.resolve(true);
  }

  /**
   * Gets the first L1 to L2 message index in the L1 to L2 message tree which is greater than or equal to `startIndex`.
   * @param l1ToL2Message - The L1 to L2 message.
   * @param startIndex - The index to start searching from.
   * @returns The index of the L1 to L2 message in the L1 to L2 message tree (undefined if not found).
   */
  getL1ToL2MessageIndex(l1ToL2Message: Fr, startIndex: bigint): Promise<bigint | undefined> {
    return Promise.resolve(this.l1ToL2Messages.getMessageIndex(l1ToL2Message, startIndex));
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks.
   * @remarks When "from" is smaller than genesis block number, blocks from the beginning are returned.
   */
  public getBlocks(from: number, limit: number): Promise<L2Block[]> {
    // Return an empty array if we are outside of range
    if (limit < 1) {
      return Promise.reject(new Error(`Invalid limit: ${limit}`));
    }

    const fromIndex = Math.max(from - INITIAL_L2_BLOCK_NUM, 0);
    if (fromIndex >= this.l2Blocks.length) {
      return Promise.resolve([]);
    }

    const toIndex = fromIndex + limit;
    return Promise.resolve(this.l2Blocks.slice(fromIndex, toIndex));
  }

  /**
   * Gets a tx effect.
   * @param txHash - The txHash of the tx effect.
   * @returns The requested tx effect.
   */
  public getTxEffect(txHash: TxHash): Promise<TxEffect | undefined> {
    const txEffect = this.txEffects.find(tx => tx.txHash.equals(txHash));
    return Promise.resolve(txEffect);
  }

  /**
   * Gets a receipt of a settled tx.
   * @param txHash - The hash of a tx we try to get the receipt for.
   * @returns The requested tx receipt (or undefined if not found).
   */
  public getSettledTxReceipt(txHash: TxHash): Promise<TxReceipt | undefined> {
    for (const block of this.l2Blocks) {
      for (const txEffect of block.body.txEffects) {
        if (txEffect.txHash.equals(txHash)) {
          return Promise.resolve(
            new TxReceipt(
              txHash,
              TxReceipt.statusFromRevertCode(txEffect.revertCode),
              '',
              txEffect.transactionFee.toBigInt(),
              block.hash().toBuffer(),
              block.number,
            ),
          );
        }
      }
    }
    return Promise.resolve(undefined);
  }

  /**
   * Gets L1 to L2 message (to be) included in a given block.
   * @param blockNumber - L2 block number to get messages for.
   * @returns The L1 to L2 messages/leaves of the messages subtree (throws if not found).
   */
  getL1ToL2Messages(blockNumber: bigint): Promise<Fr[]> {
    return Promise.resolve(this.l1ToL2Messages.getMessages(blockNumber));
  }

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  getLogs<TLogType extends LogType>(
    from: number,
    limit: number,
    logType: TLogType,
  ): Promise<L2BlockL2Logs<FromLogType<TLogType>>[]> {
    if (from < INITIAL_L2_BLOCK_NUM || limit < 1) {
      throw new Error(`Invalid limit: ${limit}`);
    }
    const logs = (() => {
      switch (logType) {
        case LogType.ENCRYPTED:
          return this.encryptedLogsPerBlock;
        case LogType.NOTEENCRYPTED:
          return this.noteEncryptedLogsPerBlock;
        case LogType.UNENCRYPTED:
        default:
          return this.unencryptedLogsPerBlock;
      }
    })() as L2BlockL2Logs<FromLogType<TLogType>>[];

    if (from > logs.length) {
      return Promise.resolve([]);
    }
    const startIndex = from - INITIAL_L2_BLOCK_NUM;
    const endIndex = startIndex + limit;
    return Promise.resolve(logs.slice(startIndex, endIndex));
  }

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   * @remarks Works by doing an intersection of all params in the filter.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    let txHash: TxHash | undefined;
    let fromBlockIndex = 0;
    let toBlockIndex = this.unencryptedLogsPerBlock.length;
    let txIndexInBlock = 0;
    let logIndexInTx = 0;

    if (filter.afterLog) {
      // Continuation parameter is set --> tx hash is ignored
      if (filter.fromBlock == undefined || filter.fromBlock <= filter.afterLog.blockNumber) {
        fromBlockIndex = filter.afterLog.blockNumber - INITIAL_L2_BLOCK_NUM;
        txIndexInBlock = filter.afterLog.txIndex;
        logIndexInTx = filter.afterLog.logIndex + 1; // We want to start from the next log
      } else {
        fromBlockIndex = filter.fromBlock - INITIAL_L2_BLOCK_NUM;
      }
    } else {
      txHash = filter.txHash;

      if (filter.fromBlock !== undefined) {
        fromBlockIndex = filter.fromBlock - INITIAL_L2_BLOCK_NUM;
      }
    }

    if (filter.toBlock !== undefined) {
      toBlockIndex = filter.toBlock - INITIAL_L2_BLOCK_NUM;
    }

    // Ensure the indices are within block array bounds
    fromBlockIndex = Math.max(fromBlockIndex, 0);
    toBlockIndex = Math.min(toBlockIndex, this.unencryptedLogsPerBlock.length);

    if (fromBlockIndex > this.unencryptedLogsPerBlock.length || toBlockIndex < fromBlockIndex || toBlockIndex <= 0) {
      return Promise.resolve({
        logs: [],
        maxLogsHit: false,
      });
    }

    const contractAddress = filter.contractAddress;

    const logs: ExtendedUnencryptedL2Log[] = [];

    for (; fromBlockIndex < toBlockIndex; fromBlockIndex++) {
      const block = this.l2Blocks[fromBlockIndex];
      const blockLogs = this.unencryptedLogsPerBlock[fromBlockIndex];
      for (; txIndexInBlock < blockLogs.txLogs.length; txIndexInBlock++) {
        const txLogs = blockLogs.txLogs[txIndexInBlock].unrollLogs();
        for (; logIndexInTx < txLogs.length; logIndexInTx++) {
          const log = txLogs[logIndexInTx];
          if (
            (!txHash || block.body.txEffects[txIndexInBlock].txHash.equals(txHash)) &&
            (!contractAddress || log.contractAddress.equals(contractAddress))
          ) {
            logs.push(new ExtendedUnencryptedL2Log(new LogId(block.number, txIndexInBlock, logIndexInTx), log));
            if (logs.length === this.maxLogs) {
              return Promise.resolve({
                logs,
                maxLogsHit: true,
              });
            }
          }
        }
        logIndexInTx = 0;
      }
      txIndexInBlock = 0;
    }

    return Promise.resolve({
      logs,
      maxLogsHit: false,
    });
  }

  /**
   * Gets the number of the latest L2 block processed.
   * @returns The number of the latest L2 block processed.
   */
  public getSynchedL2BlockNumber(): Promise<number> {
    if (this.l2Blocks.length === 0) {
      return Promise.resolve(INITIAL_L2_BLOCK_NUM - 1);
    }
    return Promise.resolve(this.l2Blocks[this.l2Blocks.length - 1].number);
  }

  public getSynchPoint(): Promise<ArchiverL1SynchPoint> {
    return Promise.resolve({
      blocksSynchedTo: this.lastL1BlockNewBlocks,
      messagesSynchedTo: this.lastL1BlockNewMessages,
    });
  }

  public addContractArtifact(address: AztecAddress, contract: ContractArtifact): Promise<void> {
    this.contractArtifacts.set(address.toString(), contract);
    return Promise.resolve();
  }

  public getContractArtifact(address: AztecAddress): Promise<ContractArtifact | undefined> {
    return Promise.resolve(this.contractArtifacts.get(address.toString()));
  }
}
