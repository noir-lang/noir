import { EthAddress } from '@aztec/foundation';
import { EthereumRpc, TransactionReceipt, TxHash } from '../eth_rpc/index.js';
import { ContractAbi } from './abi/index.js';
import { ContractTxReceipt } from './contract_tx_receipt.js';
import { SentContractTx } from './sent_contract_tx.js';

/**
 * Extends the standard contract SentContractTx class to execute a callback, which is currently used to set the
 * contract address on the original Contract instance.
 */
export class SentDeployContractTx extends SentContractTx {
  constructor(
    eth: EthereumRpc,
    contractAbi: ContractAbi,
    promise: Promise<TxHash>,
    private onDeployed: (address: EthAddress) => void,
  ) {
    super(eth, contractAbi, promise);
  }

  /**
   * Handles the transaction receipt for a contract deployment, executing a callback
   * with the deployed contract address when it is available in the receipt.
   * Extends the base class method to include the onDeployed callback execution.
   *
   * @param throwOnError - A boolean flag indicating whether to throw an error on non-zero exit status.
   * @param receipt - The transaction receipt object containing the contract address and other details.
   * @returns A Promise resolving to a ContractTxReceipt instance containing transaction receipt data.
   */
  protected async handleReceipt(throwOnError = true, receipt: TransactionReceipt): Promise<ContractTxReceipt> {
    if (receipt.contractAddress) {
      this.onDeployed(receipt.contractAddress);
    }

    return await super.handleReceipt(throwOnError, receipt);
  }
}
