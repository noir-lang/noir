import { AztecNode } from '@aztec/aztec-node';
import { InterruptableSleep } from '@aztec/foundation';
import { AccountState } from '../account_state/index.js';
import { AztecAddress, EthAddress } from '../circuits.js';
import { Database } from '../database/index.js';
import { ContractAbi } from '../noir.js';
import { TxHash } from '../tx/index.js';

export class Synchroniser {
  private runningPromise?: Promise<void>;
  private accountStates: AccountState[] = [];
  private interruptableSleep = new InterruptableSleep();
  private running = false;

  constructor(private node: AztecNode, private db: Database) {}

  public start(from = 0, take = 1, retryInterval = 10000) {
    if (this.running) {
      return;
    }

    this.running = true;

    const run = async () => {
      while (this.running) {
        try {
          const blocks = await this.node.getBlocks(from, take);

          if (!blocks.length) {
            await this.interruptableSleep.sleep(retryInterval);
            continue;
          }

          const contractAddresses = blocks
            .map(b => b.newContractData.map(d => d.aztecAddress))
            .flat()
            .map(fr => new AztecAddress(fr.toBuffer()));
          await this.db.confirmContractsDeployed(contractAddresses);

          from += blocks.length;
        } catch (err) {
          console.log(err);
          await this.interruptableSleep.sleep(retryInterval);
        }
      }
    };

    this.runningPromise = run();
  }

  public async stop() {
    this.running = false;
    this.interruptableSleep.interrupt();
    await this.runningPromise;
  }

  public async addAccount(account: AztecAddress) {
    this.accountStates.push(new AccountState(account, this.db));
    await Promise.resolve();
  }

  public getAccount(account: AztecAddress) {
    return this.accountStates.find(as => as.publicKey.equals(account));
  }

  public getAccounts() {
    return [...this.accountStates];
  }

  public async addPendingContractAbi(contractAddress: AztecAddress, portalContract: EthAddress, abi: ContractAbi) {
    await this.db.addContract(contractAddress, portalContract, abi, false);
  }

  public async getTxReceipt(txHash: TxHash) {
    const tx = await this.db.getTx(txHash);
    if (!tx) {
      return;
    }

    const account = this.getAccount(tx.from);
    if (!account) {
      throw new Error('Unauthorised account.');
    }

    return {
      txHash: tx.txHash,
      blockHash: tx.blockHash,
      blockNumber: tx.blockNumber,
      from: tx.from,
      to: tx.to,
      contractAddress: tx.contractAddress,
      error: tx.error,
      status: !tx.error,
    };
  }
}
