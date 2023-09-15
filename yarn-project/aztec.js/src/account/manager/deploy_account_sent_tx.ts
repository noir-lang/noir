import { FieldsOf } from '@aztec/circuits.js';
import { TxHash, TxReceipt } from '@aztec/types';

import { SentTx, WaitOpts, Wallet } from '../../index.js';

/** Extends a transaction receipt with a wallet instance for the newly deployed contract. */
export type DeployAccountTxReceipt = FieldsOf<TxReceipt> & {
  /** Wallet that corresponds to the newly deployed account contract. */
  wallet: Wallet;
};

/**
 * A deployment transaction for an account contract sent to the network, extending SentTx with methods to get the resulting wallet.
 */
export class DeployAccountSentTx extends SentTx {
  constructor(private wallet: Wallet, txHashPromise: Promise<TxHash>) {
    super(wallet, txHashPromise);
  }

  /**
   * Awaits for the tx to be mined and returns the contract instance. Throws if tx is not mined.
   * @param opts - Options for configuring the waiting for the tx to be mined.
   * @returns The deployed contract instance.
   */
  public async getWallet(opts?: WaitOpts): Promise<Wallet> {
    const receipt = await this.wait(opts);
    return receipt.wallet;
  }

  /**
   * Awaits for the tx to be mined and returns the receipt along with a wallet instance. Throws if tx is not mined.
   * @param opts - Options for configuring the waiting for the tx to be mined.
   * @returns The transaction receipt with the wallet for the deployed account contract.
   */
  public async wait(opts?: WaitOpts): Promise<DeployAccountTxReceipt> {
    const receipt = await super.wait(opts);
    return { ...receipt, wallet: this.wallet };
  }
}
