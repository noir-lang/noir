import { AztecAddress, Fr, PublicKey } from '@aztec/circuits.js';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { InterruptableSleep } from '@aztec/foundation/sleep';
import { AztecNode, KeyStore, L2BlockContext, LogType, MerkleTreeId } from '@aztec/types';

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

  constructor(private node: AztecNode, private db: Database, logSuffix = '') {
    this.log = createDebugLogger('aztec:aztec_rpc_synchroniser_' + logSuffix);
  }

  /**
   * Starts the synchronisation process by fetching encrypted logs and blocks from a specified position.
   * Continuously processes the fetched data for all note processors until stopped. If there is no data
   * available, it retries after a specified interval.
   *
   * @param from - The starting position for fetching encrypted logs and blocks.
   * @param take - The number of encrypted logs and blocks to fetch in each iteration.
   * @param retryInterval - The time interval (in ms) to wait before retrying if no data is available.
   */
  public async start(from = 1, take = 1, retryInterval = 1000) {
    if (this.running) return;
    this.running = true;

    await this.initialSync();

    const run = async () => {
      while (this.running) {
        from = await this.work(from, take, retryInterval);
      }
    };

    this.runningPromise = run();
    this.log('Started');
  }

  protected async initialSync() {
    const [blockNumber, treeRoots] = await Promise.all([
      this.node.getBlockHeight(),
      Promise.resolve(this.node.getTreeRoots()),
    ]);
    this.initialSyncBlockHeight = blockNumber;
    this.synchedToBlock = this.initialSyncBlockHeight;
    await this.db.setTreeRoots(treeRoots);
  }

  protected async work(from = 1, take = 1, retryInterval = 1000): Promise<number> {
    try {
      let encryptedLogs = await this.node.getLogs(from, take, LogType.ENCRYPTED);
      if (!encryptedLogs.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return from;
      }

      let unencryptedLogs = await this.node.getLogs(from, take, LogType.UNENCRYPTED);
      if (!unencryptedLogs.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return from;
      }

      // Note: If less than `take` encrypted logs is returned, then we fetch only that number of blocks.
      const blocks = await this.node.getBlocks(from, encryptedLogs.length);
      if (!blocks.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return from;
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
      await this.setTreeRootsFromBlock(latestBlock);

      this.log(
        `Forwarding ${encryptedLogs.length} encrypted logs and blocks to ${this.noteProcessors.length} note processors`,
      );
      for (const noteProcessor of this.noteProcessors) {
        await noteProcessor.process(blockContexts, encryptedLogs);
      }

      await this.updateBlockInfoInBlockTxs(blockContexts);

      from += encryptedLogs.length;
      this.synchedToBlock = latestBlock.block.number;
      return from;
    } catch (err) {
      this.log(err);
      await this.interruptableSleep.sleep(retryInterval);
      return from;
    }
  }

  private async setTreeRootsFromBlock(latestBlock: L2BlockContext) {
    const { block } = latestBlock;
    if (block.number < this.initialSyncBlockHeight) return;

    const roots: Record<MerkleTreeId, Fr> = {
      [MerkleTreeId.CONTRACT_TREE]: block.endContractTreeSnapshot.root,
      [MerkleTreeId.PRIVATE_DATA_TREE]: block.endPrivateDataTreeSnapshot.root,
      [MerkleTreeId.NULLIFIER_TREE]: block.endNullifierTreeSnapshot.root,
      [MerkleTreeId.PUBLIC_DATA_TREE]: block.endPublicDataTreeRoot,
      [MerkleTreeId.L1_TO_L2_MESSAGES_TREE]: block.endL1ToL2MessageTreeSnapshot.root,
      [MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE]: block.endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot.root,
      [MerkleTreeId.CONTRACT_TREE_ROOTS_TREE]: block.endTreeOfHistoricContractTreeRootsSnapshot.root,
      [MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE]: block.endTreeOfHistoricPrivateDataTreeRootsSnapshot.root,
    };
    await this.db.setTreeRoots(roots);
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
   * @param address - The address for the account.
   * @param keyStore - The key store.
   * @returns A promise that resolves once the account is added to the Synchroniser.
   */
  public addAccount(publicKey: PublicKey, address: AztecAddress, keyStore: KeyStore) {
    const processor = this.noteProcessors.find(x => x.publicKey.equals(publicKey));
    if (processor) {
      return;
    }
    this.noteProcessors.push(new NoteProcessor(publicKey, address, keyStore, this.db, this.node));
  }

  /**
   * Returns true if the account specified by the given address is synched to the latest block
   * @param account - The aztec address for which to query the sync status
   * @returns True if the account is fully synched, false otherwise
   */
  public async isAccountSynchronised(account: AztecAddress) {
    const result = await this.db.getPublicKeyAndPartialAddress(account);
    if (!result) {
      return false;
    }
    const publicKey = result[0];
    const processor = this.noteProcessors.find(x => x.publicKey.equals(publicKey));
    if (!processor) {
      return false;
    }
    return await processor.isSynchronised();
  }

  /**
   * Return true if the top level block synchronisation is up to date
   * This indicates that blocks and transactions are synched even if notes are not
   * @returns True if there are no outstanding blocks to be synched
   */
  public async isSynchronised() {
    const latest = await this.node.getBlockHeight();
    return latest <= this.synchedToBlock;
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
