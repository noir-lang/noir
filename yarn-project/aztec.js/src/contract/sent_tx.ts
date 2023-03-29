import { AztecRPCClient, TxReceipt, TxHash, TxStatus } from '@aztec/aztec-rpc';
import { retryUntil } from '@aztec/foundation';

export class SentTx {
  constructor(private arc: AztecRPCClient, private txHashPromise: Promise<TxHash>) {}

  public async getTxHash() {
    return await this.txHashPromise;
  }

  public async getReceipt(): Promise<TxReceipt> {
    const txHash = await this.getTxHash();
    return await this.arc.getTxReceipt(txHash);
  }

  public async isMined(timeout = 0, interval = 1): Promise<boolean> {
    const txHash = await this.getTxHash();
    const receipt = await retryUntil(
      async () => {
        const txReceipt = await this.arc.getTxReceipt(txHash);
        return txReceipt.status != TxStatus.PENDING ? txReceipt : undefined;
      },
      'isMined',
      timeout,
      interval,
    );
    return receipt.status === TxStatus.MINED;
  }
}
