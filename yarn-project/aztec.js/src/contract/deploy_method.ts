import { type FunctionCall, type Tx, type TxExecutionRequest } from '@aztec/circuit-types';
import {
  AztecAddress,
  computePartialAddress,
  getContractClassFromArtifact,
  getContractInstanceFromDeployParams,
} from '@aztec/circuits.js';
import { type ContractArtifact, type FunctionArtifact, getInitializer } from '@aztec/foundation/abi';
import { type Fr } from '@aztec/foundation/fields';
import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { type Wallet } from '../account/index.js';
import { deployInstance } from '../deployment/deploy_instance.js';
import { registerContractClass } from '../deployment/register_class.js';
import { type ExecutionRequestInit } from '../entrypoint/entrypoint.js';
import { BaseContractInteraction, type SendMethodOptions } from './base_contract_interaction.js';
import { type Contract } from './contract.js';
import { type ContractBase } from './contract_base.js';
import { ContractFunctionInteraction } from './contract_function_interaction.js';
import { DeploySentTx } from './deploy_sent_tx.js';

/**
 * Options for deploying a contract on the Aztec network.
 * Allows specifying a contract address salt, and additional send method options.
 */
// docs:start:deploy_options
export type DeployOptions = {
  /** An optional salt value used to deterministically calculate the contract address. */
  contractAddressSalt?: Fr;
  /** Set to true to *not* include the sender in the address computation. */
  universalDeploy?: boolean;
  /** Skip contract class registration. */
  skipClassRegistration?: boolean;
  /** Skip public deployment, instead just privately initialize the contract. */
  skipPublicDeployment?: boolean;
  /** Skip contract initialization. */
  skipInitialization?: boolean;
} & SendMethodOptions;
// docs:end:deploy_options
// TODO(@spalladino): Add unit tests for this class!

/**
 * Contract interaction for deployment. Handles class registration, public instance deployment,
 * and initialization of the contract. Extends the BaseContractInteraction class.
 */
export class DeployMethod<TContract extends ContractBase = Contract> extends BaseContractInteraction {
  /** The contract instance to be deployed. */
  private instance?: ContractInstanceWithAddress = undefined;

  /** Constructor function to call. */
  private constructorArtifact: FunctionArtifact | undefined;

  /** Cached call to request() */
  private functionCalls?: ExecutionRequestInit;

  constructor(
    private publicKeysHash: Fr,
    wallet: Wallet,
    private artifact: ContractArtifact,
    private postDeployCtor: (address: AztecAddress, wallet: Wallet) => Promise<TContract>,
    private args: any[] = [],
    constructorNameOrArtifact?: string | FunctionArtifact,
  ) {
    super(wallet);
    this.constructorArtifact = getInitializer(artifact, constructorNameOrArtifact);
  }

  /**
   * Create a contract deployment transaction, given the deployment options.
   * This function internally calls `request()` and `sign()` methods to prepare
   * the transaction for deployment. The resulting signed transaction can be
   * later sent using the `send()` method.
   *
   * @param options - An object containing optional deployment settings, contractAddressSalt, and from.
   * @returns A Promise resolving to an object containing the signed transaction data and other relevant information.
   */
  public async create(options: DeployOptions = {}): Promise<TxExecutionRequest> {
    if (!this.txRequest) {
      this.txRequest = await this.wallet.createTxExecutionRequest(await this.request(options));
    }
    return this.txRequest;
  }

  // REFACTOR: Having a `request` method with different semantics than the ones in the other
  // derived ContractInteractions is confusing. We should unify the flow of all ContractInteractions.

  /**
   * Returns an array of function calls that represent this operation. Useful as a building
   * block for constructing batch requests.
   * @param options - Deployment options.
   * @returns An array of function calls.
   * @remarks This method does not have the same return type as the `request` in the ContractInteraction object,
   * it returns a promise for an array instead of a function call directly.
   */
  public async request(options: DeployOptions = {}): Promise<ExecutionRequestInit> {
    if (!this.functionCalls) {
      // TODO: Should we add the contracts to the DB here, or once the tx has been sent or mined?
      // Note that we need to run this registerContract here so it's available when computeFeeOptionsFromEstimatedGas
      // runs, since it needs the contract to have been registered in order to estimate gas for its initialization,
      // in case the initializer is public. This hints at the need of having "transient" contracts scoped to a
      // simulation, so we can run the simulation with a set of contracts, but only "commit" them to the wallet
      // once this tx has gone through.
      await this.wallet.registerContract({ artifact: this.artifact, instance: this.getInstance(options) });

      const deployment = await this.getDeploymentFunctionCalls(options);
      const bootstrap = await this.getInitializeFunctionCalls(options);

      if (deployment.calls.length + bootstrap.calls.length === 0) {
        throw new Error(`No function calls needed to deploy contract ${this.artifact.name}`);
      }

      const request = {
        calls: [...deployment.calls, ...bootstrap.calls],
        authWitnesses: [...(deployment.authWitnesses ?? []), ...(bootstrap.authWitnesses ?? [])],
        packedArguments: [...(deployment.packedArguments ?? []), ...(bootstrap.packedArguments ?? [])],
        fee: options.fee,
      };

      if (options.estimateGas) {
        // Why do we call this seemingly idempotent getter method here, without using its return value?
        // This call pushes a capsule required for contract class registration under the hood. And since
        // capsules are a stack, when we run the simulation for estimating gas, we consume the capsule
        // that was meant for the actual call. So we need to push it again here. Hopefully this design
        // will go away soon.
        await this.getDeploymentFunctionCalls(options);
        request.fee = await this.getFeeOptionsFromEstimatedGas(request);
      }

      this.functionCalls = request;
    }

    return this.functionCalls;
  }

  /**
   * Register this contract in the PXE and returns the Contract object.
   * @param options - Deployment options.
   */
  public async register(options: DeployOptions = {}): Promise<TContract> {
    const instance = this.getInstance(options);
    await this.wallet.registerContract({ artifact: this.artifact, instance });
    return this.postDeployCtor(instance.address, this.wallet);
  }

  /**
   * Returns calls for registration of the class and deployment of the instance, depending on the provided options.
   * @param options - Deployment options.
   * @returns A function call array with potentially requests to the class registerer and instance deployer.
   */
  protected async getDeploymentFunctionCalls(options: DeployOptions = {}): Promise<ExecutionRequestInit> {
    const calls: FunctionCall[] = [];

    // Set contract instance object so it's available for populating the DeploySendTx object
    const instance = this.getInstance(options);

    // Obtain contract class from artifact and check it matches the reported one by the instance.
    // TODO(@spalladino): We're unnecessarily calculating the contract class multiple times here.
    const contractClass = getContractClassFromArtifact(this.artifact);
    if (!instance.contractClassId.equals(contractClass.id)) {
      throw new Error(
        `Contract class mismatch when deploying contract: got ${instance.contractClassId.toString()} from instance and ${contractClass.id.toString()} from artifact`,
      );
    }

    // Register the contract class if it hasn't been published already.
    if (!options.skipClassRegistration) {
      if (await this.wallet.isContractClassPubliclyRegistered(contractClass.id)) {
        this.log.debug(
          `Skipping registration of already registered contract class ${contractClass.id.toString()} for ${instance.address.toString()}`,
        );
      } else {
        this.log.info(
          `Creating request for registering contract class ${contractClass.id.toString()} as part of deployment for ${instance.address.toString()}`,
        );
        calls.push((await registerContractClass(this.wallet, this.artifact)).request());
      }
    }

    // Deploy the contract via the instance deployer.
    if (!options.skipPublicDeployment) {
      calls.push(deployInstance(this.wallet, instance).request());
    }

    return {
      calls,
    };
  }

  /**
   * Returns the calls necessary to initialize the contract.
   * @param options - Deployment options.
   * @returns - An array of function calls.
   */
  protected getInitializeFunctionCalls(options: DeployOptions): Promise<ExecutionRequestInit> {
    const { address } = this.getInstance(options);
    const calls: FunctionCall[] = [];
    if (this.constructorArtifact && !options.skipInitialization) {
      const constructorCall = new ContractFunctionInteraction(
        this.wallet,
        address,
        this.constructorArtifact,
        this.args,
      );
      calls.push(constructorCall.request());
    }
    return Promise.resolve({
      calls,
    });
  }

  /**
   * Send the contract deployment transaction using the provided options.
   * This function extends the 'send' method from the ContractFunctionInteraction class,
   * allowing us to send a transaction specifically for contract deployment.
   *
   * @param options - An object containing various deployment options such as contractAddressSalt and from.
   * @returns A SentTx object that returns the receipt and the deployed contract instance.
   */
  public override send(options: DeployOptions = {}): DeploySentTx<TContract> {
    const txHashPromise = super.send(options).getTxHash();
    const instance = this.getInstance(options);
    this.log.debug(
      `Sent deployment tx of ${this.artifact.name} contract with deployment address ${instance.address.toString()}`,
    );
    return new DeploySentTx(this.wallet, txHashPromise, this.postDeployCtor, instance);
  }

  /**
   * Builds the contract instance to be deployed and returns it.
   *
   * @param options - An object containing various deployment options.
   * @returns An instance object.
   */
  public getInstance(options: DeployOptions = {}): ContractInstanceWithAddress {
    if (!this.instance) {
      this.instance = getContractInstanceFromDeployParams(this.artifact, {
        constructorArgs: this.args,
        salt: options.contractAddressSalt,
        publicKeysHash: this.publicKeysHash,
        constructorArtifact: this.constructorArtifact,
        deployer: options.universalDeploy ? AztecAddress.ZERO : this.wallet.getAddress(),
      });
    }
    return this.instance;
  }

  /**
   * Prove the request.
   * @param options - Deployment options.
   * @returns The proven tx.
   */
  public override prove(options: DeployOptions): Promise<Tx> {
    return super.prove(options);
  }

  /**
   * Estimates gas cost for this deployment operation.
   * @param options - Options.
   */
  public override estimateGas(options?: Omit<DeployOptions, 'estimateGas' | 'skipPublicSimulation'>) {
    return super.estimateGas(options);
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
