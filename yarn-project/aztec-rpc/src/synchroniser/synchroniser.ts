import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { L2BlockContext, TxHash } from '@aztec/types';
import { AccountState } from '../account_state/index.js';
import { Database, TxDao } from '../database/index.js';
import { InterruptableSleep } from '@aztec/foundation/sleep';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { MerkleTreeId } from '@aztec/types';
import { Fr } from '@aztec/circuits.js';

export class Synchroniser {
  private runningPromise?: Promise<void>;
  private accountStates: AccountState[] = [];
  private interruptableSleep = new InterruptableSleep();
  private running = false;
  private initialSyncBlockHeight = 0;

  constructor(
    private node: AztecNode,
    private db: Database,
    private log = createDebugLogger('aztec:aztec_rpc_synchroniser'),
  ) {}

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
    const [blockNumber, treeRoots] = await Promise.all([this.node.getBlockHeight(), this.node.getTreeRoots()]);
    this.initialSyncBlockHeight = blockNumber;
    await this.db.setTreeRoots(treeRoots);
  }

  protected async work(from = 1, take = 1, retryInterval = 1000): Promise<number> {
    try {
      let unverifiedData = await this.node.getUnverifiedData(from, take);
      if (!unverifiedData.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return from;
      }

      // Note: If less than `take` unverified data is returned, then I fetch only that number of blocks.
      const blocks = await this.node.getBlocks(from, unverifiedData.length);
      if (!blocks.length) {
        await this.interruptableSleep.sleep(retryInterval);
        return from;
      }

      if (blocks.length !== unverifiedData.length) {
        // "Trim" the unverified data to match the number of blocks.
        unverifiedData = unverifiedData.slice(0, blocks.length);
      }

      // Wrap blocks in block contexts.
      const blockContexts = blocks.map(block => new L2BlockContext(block));

      // Update latest tree roots from the most recent block
      const latestBlock = blockContexts[blockContexts.length - 1];
      await this.setTreeRootsFromBlock(latestBlock);

      this.log(
        `Forwarding ${unverifiedData.length} unverified data and blocks to ${this.accountStates.length} account states`,
      );
      for (const accountState of this.accountStates) {
        await accountState.process(blockContexts, unverifiedData);
      }

      from += unverifiedData.length;
      return from;
    } catch (err) {
      console.log(err);
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

  public async stop() {
    this.running = false;
    this.interruptableSleep.interrupt();
    await this.runningPromise;
    this.log('Stopped');
  }

  public async addAccount(privKey: Buffer) {
    const accountState = new AccountState(privKey, this.db, this.node, await Grumpkin.new());
    this.accountStates.push(accountState);
    await Promise.resolve();
  }

  public getAccount(account: AztecAddress) {
    return this.accountStates.find(as => as.getPublicKey().toAddress().equals(account));
  }

  public getAccounts() {
    return [...this.accountStates];
  }

  public async getTxByHash(txHash: TxHash): Promise<TxDao> {
    const tx = await this.db.getTx(txHash);
    if (!tx) {
      throw new Error('Transaction not found in RPC database');
    }

    const account = this.getAccount(tx.from);
    if (!account) {
      throw new Error('Unauthorised account.');
    }

    return tx;
  }
}
