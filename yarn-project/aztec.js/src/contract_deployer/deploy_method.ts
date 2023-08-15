import {
  ContractDeploymentData,
  FunctionData,
  PartialAddress,
  TxContext,
  getContractDeploymentInfo,
} from '@aztec/circuits.js';
import { ContractAbi, FunctionAbi, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { AztecRPC, PackedArguments, PublicKey, Tx, TxExecutionRequest } from '@aztec/types';

import { BaseContractInteraction } from '../contract/base_contract_interaction.js';
import { Contract, ContractBase, SendMethodOptions } from '../contract/index.js';
import { DeploySentTx } from './deploy_sent_tx.js';

/**
 * Options for deploying a contract on the Aztec network.
 * Allows specifying a portal contract, contract address salt, and additional send method options.
 */
export interface DeployOptions extends SendMethodOptions {
  /**
   * The Ethereum address of the Portal contract.
   */
  portalContract?: EthAddress;
  /**
   * An optional salt value used to deterministically calculate the contract address.
   */
  contractAddressSalt?: Fr;
}

/**
 * Creates a TxRequest from a contract ABI, for contract deployment.
 * Extends the ContractFunctionInteraction class.
 */
export class DeployMethod<TContract extends ContractBase = Contract> extends BaseContractInteraction {
  /** The partially computed contract address. Known after creation of the deployment transaction. */
  public partialAddress?: PartialAddress = undefined;

  /** The complete contract address. */
  public completeContractAddress?: AztecAddress = undefined;

  /** Constructor function to call. */
  private constructorAbi: FunctionAbi;

  constructor(private publicKey: PublicKey, private arc: AztecRPC, private abi: ContractAbi, private args: any[] = []) {
    super(arc);
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) throw new Error('Cannot find constructor in the ABI.');
    this.constructorAbi = constructorAbi;
  }

  /**
   * Create a contract deployment transaction, given the deployment options.
   * This function internally calls `request()` and `sign()` methods to prepare
   * the transaction for deployment. The resulting signed transaction can be
   * later sent using the `send()` method.
   *
   * @param options - An object containing optional deployment settings, including portalContract, contractAddressSalt, and from.
   * @returns A Promise resolving to an object containing the signed transaction data and other relevant information.
   */
  public async create(options: DeployOptions = {}) {
    const portalContract = options.portalContract ?? EthAddress.ZERO;
    const contractAddressSalt = options.contractAddressSalt ?? Fr.random();

    const { chainId, version } = await this.rpc.getNodeInfo();

    const { completeAddress, constructorHash, functionTreeRoot } = await getContractDeploymentInfo(
      this.abi,
      this.args,
      contractAddressSalt,
      this.publicKey,
    );

    const contractDeploymentData = new ContractDeploymentData(
      this.publicKey,
      constructorHash,
      functionTreeRoot,
      contractAddressSalt,
      portalContract,
    );

    const txContext = new TxContext(false, false, true, contractDeploymentData, new Fr(chainId), new Fr(version));
    const args = encodeArguments(this.constructorAbi, this.args);
    const functionData = FunctionData.fromAbi(this.constructorAbi);
    const execution = { args, functionData, to: completeAddress.address };
    const packedArguments = await PackedArguments.fromArgs(execution.args);

    const txRequest = TxExecutionRequest.from({
      origin: execution.to,
      functionData: execution.functionData,
      argsHash: packedArguments.hash,
      txContext,
      packedArguments: [packedArguments],
    });

    this.txRequest = txRequest;
    this.partialAddress = completeAddress.partialAddress;
    this.completeContractAddress = completeAddress.address;

    // TODO: Should we add the contracts to the DB here, or once the tx has been sent or mined?
    await this.rpc.addContracts([{ abi: this.abi, address: completeAddress.address, portalContract }]);

    return this.txRequest;
  }

  /**
   * Send the contract deployment transaction using the provided options.
   * This function extends the 'send' method from the ContractFunctionInteraction class,
   * allowing us to send a transaction specifically for contract deployment.
   *
   * @param options - An object containing various deployment options such as portalContract, contractAddressSalt, and from.
   * @returns A SentTx object that returns the receipt and the deployed contract instance.
   */
  public send(options: DeployOptions = {}): DeploySentTx<TContract> {
    const txHashPromise = super.send(options).getTxHash();
    return new DeploySentTx(this.abi, this.arc, txHashPromise);
  }

  /**
   * Simulate the request.
   * @param options - Deployment options.
   * @returns The simulated tx.
   */
  public simulate(options: DeployOptions): Promise<Tx> {
    return super.simulate(options);
  }
}
