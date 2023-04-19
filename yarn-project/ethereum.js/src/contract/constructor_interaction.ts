import { EthAddress } from '@aztec/foundation';
import { EthereumRpc, SentTx } from '../eth_rpc/index.js';
import { ContractAbi, ContractFunctionEntry } from './abi/index.js';
import { SentDeployContractTx } from './sent_deploy_contract_tx.js';
import { Options, SendOptions, FunctionInteraction } from './function_interaction.js';

/**
 * Extends the plain FunctionInteraction to provide the extended abi encoding for constructor calls (deployments).
 */
export class ConstructorInteraction extends FunctionInteraction {
  constructor(
    eth: EthereumRpc,
    contractEntry: ContractFunctionEntry,
    contractAbi: ContractAbi,
    private deployData: Buffer,
    args: any[] = [],
    defaultOptions: Options = {},
    private onDeployed: (address: EthAddress) => void = x => x,
  ) {
    super(eth, contractEntry, contractAbi, undefined, args, defaultOptions);
  }

  /**
   * Sends a transaction with the encoded contract bytecode and constructor arguments, creating a new deployment of the contract.
   * Returns a SentDeployContractTx instance that can be used to track the transaction status and retrieve the deployed contract address.
   * The 'options' parameter can be used to customize the transaction, such as specifying gas price, gas limit, or value to send.
   *
   * @param options - An object containing optional parameters for customizing the transaction.
   * @returns A SentDeployContractTx instance representing the sent transaction.
   */
  public send(options: SendOptions): SentTx {
    const sentTx = super.send(options);
    return new SentDeployContractTx(this.eth, this.contractAbi, sentTx.getTxHash(), this.onDeployed);
  }

  /**
   * Encodes the ABI (Application Binary Interface) for the function interaction with the provided arguments.
   *  The encoded ABI is a serialized representation of the function's signature and its arguments, which can be used by the Ethereum client to process the method call or transaction.
   * This is useful for encoding contract function calls when interacting with the Ethereum blockchain.
   * @returns The contract bytecode concatenated with the abi encoded constructor arguments.
   */
  public encodeABI() {
    return Buffer.concat([this.deployData, this.contractEntry.encodeParameters(this.args)]);
  }
}
