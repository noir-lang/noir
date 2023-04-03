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

  public send(options: SendOptions): SentTx {
    const sentTx = super.send(options);
    return new SentDeployContractTx(this.eth, this.contractAbi, sentTx.getTxHash(), this.onDeployed);
  }

  /**
   * @returns The contract bytecode concatenated with the abi encoded constructor arguments.
   */
  public encodeABI() {
    return Buffer.concat([this.deployData, this.contractEntry.encodeParameters(this.args)]);
  }
}
