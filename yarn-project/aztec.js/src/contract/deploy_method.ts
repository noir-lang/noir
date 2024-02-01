import { PXE, PackedArguments, PublicKey, Tx, TxExecutionRequest } from '@aztec/circuit-types';
import {
  AztecAddress,
  ContractDeploymentData,
  FunctionData,
  TxContext,
  computePartialAddress,
  getContractInstanceFromDeployParams,
} from '@aztec/circuits.js';
import { ContractArtifact, FunctionArtifact, encodeArguments } from '@aztec/foundation/abi';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { ContractInstanceWithAddress } from '@aztec/types/contracts';

import { Wallet } from '../account/index.js';
import { BaseContractInteraction, SendMethodOptions } from './base_contract_interaction.js';
import { type Contract } from './contract.js';
import { ContractBase } from './contract_base.js';
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
  /** The contract instance to be deployed. */
  public instance?: ContractInstanceWithAddress = undefined;

  /** Constructor function to call. */
  private constructorArtifact: FunctionArtifact;

  constructor(
    private publicKey: PublicKey,
    protected pxe: PXE,
    private artifact: ContractArtifact,
    private postDeployCtor: (address: AztecAddress, wallet: Wallet) => Promise<TContract>,
    private args: any[] = [],
  ) {
    super(pxe);
    const constructorArtifact = artifact.functions.find(f => f.name === 'constructor');
    if (!constructorArtifact) {
      throw new Error('Cannot find constructor in the artifact.');
    }
    this.constructorArtifact = constructorArtifact;
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

    const deployParams = [this.artifact, this.args, contractAddressSalt, this.publicKey, portalContract] as const;
    const instance = getContractInstanceFromDeployParams(...deployParams);
    const address = instance.address;

    const contractDeploymentData = new ContractDeploymentData(
      this.publicKey,
      instance.initializationHash,
      instance.contractClassId,
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
    const args = encodeArguments(this.constructorArtifact, this.args);
    const functionData = FunctionData.fromAbi(this.constructorArtifact);
    const execution = { args, functionData, to: address };
    const packedArguments = PackedArguments.fromArgs(execution.args);

    const txRequest = TxExecutionRequest.from({
      origin: execution.to,
      functionData: execution.functionData,
      argsHash: packedArguments.hash,
      txContext,
      packedArguments: [packedArguments],
      authWitnesses: [],
    });

    this.txRequest = txRequest;
    this.instance = instance;

    // TODO: Should we add the contracts to the DB here, or once the tx has been sent or mined?
    await this.pxe.addContracts([{ artifact: this.artifact, instance }]);

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
    return new DeploySentTx(this.pxe, txHashPromise, this.postDeployCtor, this.instance!);
  }

  /**
   * Simulate the request.
   * @param options - Deployment options.
   * @returns The simulated tx.
   */
  public simulate(options: DeployOptions): Promise<Tx> {
    return super.simulate(options);
  }

  /** Return this deployment address. */
  public get address() {
    return this.instance?.address;
  }

  /** Returns the partial address for this deployment. */
  public get partialAddress() {
    return this.instance && computePartialAddress(this.instance);
  }
}
