import { type PXE, type TxHash, type TxReceipt } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { type FieldsOf } from '@aztec/foundation/types';
import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { type Wallet } from '../account/index.js';
import { type Contract } from './contract.js';
import { type ContractBase } from './contract_base.js';
import { SentTx, type WaitOpts } from './sent_tx.js';

/** Options related to waiting for a deployment tx. */
export type DeployedWaitOpts = WaitOpts & {
  /** Wallet to use for creating a contract instance. Uses the one set in the deployer constructor if not set. */
  wallet?: Wallet;
};

/** Extends a transaction receipt with a contract instance that represents the newly deployed contract. */
export type DeployTxReceipt<TContract extends ContractBase = Contract> = FieldsOf<TxReceipt> & {
  /** Instance of the newly deployed contract. */
  contract: TContract;
};

/**
 * A contract deployment transaction sent to the network, extending SentTx with methods to create a contract instance.
 */
export class DeploySentTx<TContract extends Contract = Contract> extends SentTx {
  constructor(
    wallet: PXE | Wallet,
    txHashPromise: Promise<TxHash>,
    private postDeployCtor: (address: AztecAddress, wallet: Wallet) => Promise<TContract>,
    /** The deployed contract instance */
    public instance: ContractInstanceWithAddress,
  ) {
    super(wallet, txHashPromise);
  }

  /**
   * Awaits for the tx to be mined and returns the contract instance. Throws if tx is not mined.
   * @param opts - Options for configuring the waiting for the tx to be mined.
   * @returns The deployed contract instance.
   */
  public async deployed(opts?: DeployedWaitOpts): Promise<TContract> {
    const receipt = await this.wait(opts);
    return receipt.contract;
  }

  /**
   * Awaits for the tx to be mined and returns the receipt along with a contract instance. Throws if tx is not mined.
   * @param opts - Options for configuring the waiting for the tx to be mined.
   * @returns The transaction receipt with the deployed contract instance.
   */
  public async wait(opts?: DeployedWaitOpts): Promise<DeployTxReceipt<TContract>> {
    const receipt = await super.wait(opts);
    const contract = await this.getContractObject(opts?.wallet);
    return { ...receipt, contract };
  }

  protected getContractObject(wallet?: Wallet): Promise<TContract> {
    const isWallet = (pxe: PXE | Wallet): pxe is Wallet => !!(pxe as Wallet).createTxExecutionRequest;
    const contractWallet = wallet ?? (isWallet(this.pxe) && this.pxe);
    if (!contractWallet) {
      throw new Error(`A wallet is required for creating a contract instance`);
    }
    return this.postDeployCtor(this.instance.address, contractWallet) as Promise<TContract>;
  }
}
