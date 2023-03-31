import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { AztecAddress } from '@aztec/circuits.js';
import { createDebugLogger, InterruptableSleep } from '@aztec/foundation';
import { TxHash } from '@aztec/tx';
import { AccountState } from '../account_state/index.js';
import { Database, TxDao } from '../database/index.js';

export class Synchroniser {
  private runningPromise?: Promise<void>;
  private accountStates: AccountState[] = [];
  private interruptableSleep = new InterruptableSleep();
  private running = false;

  constructor(
    private node: AztecNode,
    private db: Database,
    private bbWasm: BarretenbergWasm,
    private log = createDebugLogger('aztec:aztec_rpc_synchroniser'),
  ) {}

  public start(from = 1, take = 1, retryInterval = 1000) {
    if (this.running) {
      return;
    }

    this.running = true;

    let fromUnverifiedData = from;

    const run = async () => {
      while (this.running) {
        try {
          const unverifiedData = await this.node.getUnverifiedData(fromUnverifiedData, take);

          if (!unverifiedData.length) {
            await this.interruptableSleep.sleep(retryInterval);
            continue;
          }

          this.log(`Forwarded ${unverifiedData.length} unverified data to ${this.accountStates.length} account states`);
          for (const accountState of this.accountStates) {
            await accountState.processUnverifiedData(unverifiedData, fromUnverifiedData, take);
          }

          fromUnverifiedData += unverifiedData.length;
        } catch (err) {
          console.log(err);
          await this.interruptableSleep.sleep(retryInterval);
        }
      }
    };

    this.runningPromise = run();
    this.log('Started');
  }

  public async stop() {
    this.running = false;
    this.interruptableSleep.interrupt();
    await this.runningPromise;
    this.log('Stopped');
  }

  public async addAccount(privKey: Buffer) {
    this.accountStates.push(new AccountState(privKey, this.db, this.node, new Grumpkin(this.bbWasm)));
    await Promise.resolve();
  }

  public getAccount(account: AztecAddress) {
    return this.accountStates.find(async as => (await as.getPublicKey()).toAddress().equals(account));
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
