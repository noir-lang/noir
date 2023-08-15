import { FieldsOf } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { TxHash, TxReceipt } from '@aztec/types';

import { AztecAddress, AztecRPC, Contract, ContractBase, SentTx, WaitOpts, Wallet } from '../index.js';

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
export class DeploySentTx<TContract extends ContractBase = Contract> extends SentTx {
  constructor(private abi: ContractAbi, wallet: AztecRPC | Wallet, txHashPromise: Promise<TxHash>) {
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
    const contract = await this.getContractInstance(opts?.wallet, receipt.contractAddress);
    return { ...receipt, contract };
  }

  protected getContractInstance(wallet?: Wallet, address?: AztecAddress): Promise<TContract> {
    const isWallet = (rpc: AztecRPC | Wallet): rpc is Wallet => !!(rpc as Wallet).createTxExecutionRequest;
    const contractWallet = wallet ?? (isWallet(this.arc) && this.arc);
    if (!contractWallet) throw new Error(`A wallet is required for creating a contract instance`);
    if (!address) throw new Error(`Contract address is missing from transaction receipt`);
    return Contract.at(address, this.abi, contractWallet) as Promise<TContract>;
  }
}
