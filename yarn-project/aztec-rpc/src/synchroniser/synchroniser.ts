import { AztecAddress, CircuitsWasm, Fr, HistoricBlockData, PublicKey } from '@aztec/circuits.js';
import { computeGlobalsHash } from '@aztec/circuits.js/abis';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { InterruptableSleep } from '@aztec/foundation/sleep';
import { AztecNode, INITIAL_L2_BLOCK_NUM, KeyStore, L2BlockContext, LogType } from '@aztec/types';

import { Database, TxDao } from '../database/index.js';
import { NoteProcessor } from '../note_processor/index.js';

/**
 * The Synchroniser class manages the synchronization of note processors and interacts with the Aztec node
 * to obtain encrypted logs, blocks, and other necessary information for the accounts.
 * It provides methods to start or stop the synchronization process, add new accounts, retrieve account
 * details, and fetch transactions by hash. The Synchroniser ensures that it maintains the note processors
 * in sync with the blockchain while handling retries and errors gracefully.
 */
export class Synchroniser {
  private runningPromise?: Promise<void>;
  private noteProcessors: NoteProcessor[] = [];
  private interruptableSleep = new InterruptableSleep();
  private running = false;
  private initialSyncBlockHeight = 0;
  private synchedToBlock = 0;
  private log: DebugLogger;
  private noteProcessorsToCatchUp: NoteProcessor[] = [];

  constructor(private node: AztecNode, private db: Database, logSuffix = '') {
    this.log = createDebugLogger(
      logSuffix ? `aztec:aztec_rpc_synchroniser_${logSuffix}` : 'aztec:aztec_rpc_synchroniser',
    );
  }

  /**
   * Starts the synchronisation process by fetching encrypted logs and blocks from a specified position.
   * Continuously processes the fetched data for all note processors until stopped. If there is no data
   * available, it retries after a specified interval.
   *
   * @param from - The starting position for fetching encrypted logs and blocks.
   * @param limit - The maximum number of encrypted, unencrypted logs and blocks to fetch in each iteration.
   * @param retryInterval - The time interval (in ms) to wait before retrying if no data is available.
   */
  public async start(from = INITIAL_L2_BLOCK_NUM, limit = 1, retryInterval = 1000) {
    if (this.running) return;
    this.running = true;

    if (from < this.synchedToBlock + 1) {
      throw new Error(`From block ${from} is smaller than the currently synched block ${this.synchedToBlock}`);
    }
    this.synchedToBlock = from - 1;

    await this.initialSync();

    const run = async () => {
      while (this.running) {
        if (this.noteProcessorsToCatchUp.length > 0) {
          // There is a note processor that needs to catch up. We hijack the main loop to catch up the note processor.
          await this.workNoteProcessorCatchUp(limit, retryInterval);
        } else {
          // No note processor needs to catch up. We continue with the normal flow.
          await this.work(limit, retryInterval);
        }
      }
    };

    this.runningPromise = run();
    this.log('Started');
  }

  protected async initialSync() {
    const [blockNumber, historicBlockData] = await Promise.all([
      this.node.getBlockHeight(),
      Promise.resolve(this.node.getHistoricBlockData()),
    ]);
    this.initialSyncBlockHeight = blockNumber;
    this.synchedToBlock = this.initialSyncBlockHeight;
    await this.db.setHistoricBlockData(historicBlockData);
  }

  protected async work(limit = 1, retryInterval = 1000): Promise<void> {
    const from = this.synchedToBlock + 1;
    try {
      let encryptedLogs = await this.node.getLogs(from, limit, LogType.ENCRYPTED);
      if (!encryptedLogs.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return;
      }

      let unencryptedLogs = await this.node.getLogs(from, limit, LogType.UNENCRYPTED);
      if (!unencryptedLogs.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return;
      }

      // Note: If less than `limit` encrypted logs is returned, then we fetch only that number of blocks.
      const blocks = await this.node.getBlocks(from, encryptedLogs.length);
      if (!blocks.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return;
      }

      if (blocks.length !== encryptedLogs.length) {
        // "Trim" the encrypted logs to match the number of blocks.
        encryptedLogs = encryptedLogs.slice(0, blocks.length);
      }

      if (blocks.length !== unencryptedLogs.length) {
        // "Trim" the unencrypted logs to match the number of blocks.
        unencryptedLogs = unencryptedLogs.slice(0, blocks.length);
      }

      // attach logs to blocks
      blocks.forEach((block, i) => {
        block.attachLogs(encryptedLogs[i], LogType.ENCRYPTED);
        block.attachLogs(unencryptedLogs[i], LogType.UNENCRYPTED);
      });

      // Wrap blocks in block contexts.
      const blockContexts = blocks.map(block => new L2BlockContext(block));

      // Update latest tree roots from the most recent block
      const latestBlock = blockContexts[blockContexts.length - 1];
      await this.setBlockDataFromBlock(latestBlock);

      this.log(
        `Forwarding ${encryptedLogs.length} encrypted logs and blocks to ${this.noteProcessors.length} note processors`,
      );
      for (const noteProcessor of this.noteProcessors) {
        await noteProcessor.process(blockContexts, encryptedLogs);
      }

      await this.updateBlockInfoInBlockTxs(blockContexts);

      this.synchedToBlock = latestBlock.block.number;
    } catch (err) {
      this.log.error(err);
      await this.interruptableSleep.sleep(retryInterval);
    }
  }

  protected async workNoteProcessorCatchUp(limit = 1, retryInterval = 1000): Promise<void> {
    const noteProcessor = this.noteProcessorsToCatchUp[0];
    if (noteProcessor.status.syncedToBlock === this.synchedToBlock) {
      // Note processor already synched, nothing to do
      this.noteProcessorsToCatchUp.shift();
      this.noteProcessors.push(noteProcessor);
      return;
    }

    const from = noteProcessor.status.syncedToBlock + 1;
    // Ensuring that the note processor does not sync further than the main sync.
    limit = Math.min(limit, this.synchedToBlock - from + 1);

    if (limit < 1) {
      throw new Error(`Unexpected limit ${limit} for note processor catch up`);
    }

    try {
      let encryptedLogs = await this.node.getLogs(from, limit, LogType.ENCRYPTED);
      if (!encryptedLogs.length) {
        // This should never happen because this function should only be called when the note processor is lagging
        // behind main sync.
        throw new Error('No encrypted logs in processor catch up mode');
      }

      // Note: If less than `limit` encrypted logs is returned, then we fetch only that number of blocks.
      const blocks = await this.node.getBlocks(from, encryptedLogs.length);
      if (!blocks.length) {
        // This should never happen because this function should only be called when the note processor is lagging
        // behind main sync.
        throw new Error('No blocks in processor catch up mode');
      }

      if (blocks.length !== encryptedLogs.length) {
        // "Trim" the encrypted logs to match the number of blocks.
        encryptedLogs = encryptedLogs.slice(0, blocks.length);
      }

      const blockContexts = blocks.map(block => new L2BlockContext(block));

      this.log(`Forwarding ${encryptedLogs.length} encrypted logs and blocks to note processor in catch up mode`);
      await noteProcessor.process(blockContexts, encryptedLogs);

      if (noteProcessor.status.syncedToBlock === this.synchedToBlock) {
        // Note processor caught up, move it to `noteProcessors` from `noteProcessorsToCatchUp`.
        this.noteProcessorsToCatchUp.shift();
        this.noteProcessors.push(noteProcessor);
      }
    } catch (err) {
      this.log.error(err);
      await this.interruptableSleep.sleep(retryInterval);
    }
  }

  private async setBlockDataFromBlock(latestBlock: L2BlockContext) {
    const { block } = latestBlock;
    if (block.number < this.initialSyncBlockHeight) return;

    const wasm = await CircuitsWasm.get();
    const globalsHash = computeGlobalsHash(wasm, latestBlock.block.globalVariables);
    const blockData = new HistoricBlockData(
      block.endPrivateDataTreeSnapshot.root,
      block.endNullifierTreeSnapshot.root,
      block.endContractTreeSnapshot.root,
      block.endL1ToL2MessageTreeSnapshot.root,
      block.endHistoricBlocksTreeSnapshot.root,
      Fr.ZERO, // todo: private kernel vk tree root
      block.endPublicDataTreeRoot,
      globalsHash,
    );

    await this.db.setHistoricBlockData(blockData);
  }

  /**
   * Stops the synchronizer gracefully, interrupting any ongoing sleep and waiting for the current
   * iteration to complete before setting the running state to false. Once stopped, the synchronizer
   * will no longer process blocks or encrypted logs and must be restarted using the start method.
   *
   * @returns A promise that resolves when the synchronizer has successfully stopped.
   */
  public async stop() {
    this.running = false;
    this.interruptableSleep.interrupt();
    await this.runningPromise;
    this.log('Stopped');
  }

  /**
   * Add a new account to the Synchroniser with the specified private key.
   * Creates a NoteProcessor instance for the account and pushes it into the noteProcessors array.
   * The method resolves immediately after pushing the new note processor.
   *
   * @param publicKey - The public key for the account.
   * @param keyStore - The key store.
   * @returns A promise that resolves once the account is added to the Synchroniser.
   */
  public addAccount(publicKey: PublicKey, keyStore: KeyStore) {
    const processor = this.noteProcessors.find(x => x.publicKey.equals(publicKey));
    if (processor) {
      return;
    }

    this.noteProcessorsToCatchUp.push(new NoteProcessor(publicKey, keyStore, this.db, this.node));
  }

  /**
   * Checks if the specified account is synchronised.
   * @param account - The aztec address for which to query the sync status.
   * @returns True if the account is fully synched, false otherwise.
   * @remarks Checks whether all the notes from all the blocks have been processed. If it is not the case, the
   *          retrieved information from contracts might be old/stale (e.g. old token balance).
   */
  public async isAccountStateSynchronised(account: AztecAddress) {
    const completeAddress = await this.db.getCompleteAddress(account);
    if (!completeAddress) {
      return false;
    }
    const processor = this.noteProcessors.find(x => x.publicKey.equals(completeAddress.publicKey));
    if (!processor) {
      return false;
    }
    return await processor.isSynchronised();
  }

  /**
   * Checks whether all the blocks were processed (tree roots updated, txs updated with block info, etc.).
   * @returns True if there are no outstanding blocks to be synched.
   * @remarks This indicates that blocks and transactions are synched even if notes are not.
   * @remarks Compares local block height with the block height from aztec node.
   */
  public async isGlobalStateSynchronised() {
    const latest = await this.node.getBlockHeight();
    return latest <= this.synchedToBlock;
  }

  /**
   * Returns the latest block that has been synchronised by the synchronizer and each account.
   * @returns The latest block synchronised for blocks, and the latest block synched for notes for each public key being tracked.
   */
  public getSyncStatus() {
    return {
      blocks: this.synchedToBlock,
      notes: Object.fromEntries(this.noteProcessors.map(n => [n.publicKey.toString(), n.status.syncedToBlock])),
    };
  }

  /**
   * Updates the block information for all transactions in a given block context.
   * The function retrieves transaction data objects from the database using their hashes,
   * sets the block hash and block number to the corresponding values, and saves the updated
   * transaction data back to the database. If a transaction is not found in the database,
   * an informational message is logged.
   *
   * @param blockContexts - The L2BlockContext objects containing the block information and related data.
   */
  private async updateBlockInfoInBlockTxs(blockContexts: L2BlockContext[]) {
    for (const blockContext of blockContexts) {
      for (const txHash of blockContext.getTxHashes()) {
        const txDao: TxDao | undefined = await this.db.getTx(txHash);
        if (txDao !== undefined) {
          txDao.blockHash = blockContext.getBlockHash();
          txDao.blockNumber = blockContext.block.number;
          await this.db.addTx(txDao);
          this.log(`Updated tx with hash ${txHash.toString()} from block ${blockContext.block.number}`);
        } else if (!txHash.isZero()) {
          this.log(`Tx with hash ${txHash.toString()} from block ${blockContext.block.number} not found in db`);
        }
      }
    }
  }
}
