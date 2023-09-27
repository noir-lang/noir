import {
  CompleteAddress,
  ContractDeploymentData,
  FunctionData,
  TxContext,
  getContractDeploymentInfo,
} from '@aztec/circuits.js';
import { ContractAbi, FunctionAbi, encodeArguments } from '@aztec/foundation/abi';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { PXE, PackedArguments, PublicKey, Tx, TxExecutionRequest } from '@aztec/types';

import { BaseContractInteraction } from '../contract/base_contract_interaction.js';
import { Contract, ContractBase, SendMethodOptions } from '../contract/index.js';
import { DeploySentTx } from './deploy_sent_tx.js';

/**
 * Options for deploying a contract on the Aztec network.
 * Allows specifying a portal contract, contract address salt, and additional send method options.
 */
export type DeployOptions = {
  /**
   * The Ethereum address of the Portal contract.
   */
  portalContract?: EthAddress;
  /**
   * An optional salt value used to deterministically calculate the contract address.
   */
  contractAddressSalt?: Fr;
} & SendMethodOptions;

/**
 * Creates a TxRequest from a contract ABI, for contract deployment.
 * Extends the ContractFunctionInteraction class.
 */
export class DeployMethod<TContract extends ContractBase = Contract> extends BaseContractInteraction {
  /** The complete address of the contract. */
  public completeAddress?: CompleteAddress = undefined;

  /** Constructor function to call. */
  private constructorAbi: FunctionAbi;

  constructor(private publicKey: PublicKey, protected pxe: PXE, private abi: ContractAbi, private args: any[] = []) {
    super(pxe);
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

    const { chainId, protocolVersion } = await this.pxe.getNodeInfo();

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

    const txContext = new TxContext(
      false,
      false,
      true,
      contractDeploymentData,
      new Fr(chainId),
      new Fr(protocolVersion),
    );
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
      authWitnesses: [],
    });

    this.txRequest = txRequest;
    this.completeAddress = completeAddress;

    // TODO: Should we add the contracts to the DB here, or once the tx has been sent or mined?
    await this.pxe.addContracts([{ abi: this.abi, completeAddress, portalContract }]);

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
    return new DeploySentTx(this.abi, this.pxe, txHashPromise);
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
