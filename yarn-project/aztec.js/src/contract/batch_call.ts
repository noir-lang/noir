import { FunctionCall, TxExecutionRequest, Wallet } from '../index.js';
import { BaseContractInteraction } from './base_contract_interaction.js';

/** A batch of function calls to be sent as a single transaction through a wallet. */
export class BatchCall extends BaseContractInteraction {
  constructor(protected wallet: Wallet, protected calls: FunctionCall[]) {
    super(wallet);
  }

  /**
   * Create a transaction execution request that represents this batch, encoded and authenticated by the
   * user's wallet, ready to be simulated.
   * @returns A Promise that resolves to a transaction instance.
   */
  public async create(): Promise<TxExecutionRequest> {
    if (!this.txRequest) {
      this.txRequest = await this.wallet.createTxExecutionRequest(this.calls);
    }
    return this.txRequest;
  }
}
