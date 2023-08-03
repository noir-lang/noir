import {
  CircuitsWasm,
  ContractDeploymentData,
  PartialContractAddress,
  TxContext,
  getContractDeploymentInfo,
} from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { AztecRPC, ExecutionRequest, PackedArguments, PublicKey, Tx, TxExecutionRequest } from '@aztec/types';

import { BaseWallet, Wallet } from '../aztec_rpc_client/wallet.js';
import { Contract, ContractBase, ContractFunctionInteraction, SendMethodOptions } from '../contract/index.js';
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
 * Simple wallet implementation for use when deploying contracts only.
 */
class DeployerWallet extends BaseWallet {
  getAddress(): AztecAddress {
    return AztecAddress.ZERO;
  }
  async createAuthenticatedTxRequest(
    executions: ExecutionRequest[],
    txContext: TxContext,
  ): Promise<TxExecutionRequest> {
    if (executions.length !== 1) {
      throw new Error(`Deployer wallet can only run one execution at a time (requested ${executions.length})`);
    }
    const [execution] = executions;
    const wasm = await CircuitsWasm.get();
    const packedArguments = await PackedArguments.fromArgs(execution.args, wasm);
    return Promise.resolve(
      new TxExecutionRequest(execution.to, execution.functionData, packedArguments.hash, txContext, [packedArguments]),
    );
  }
}

/**
 * Creates a TxRequest from a contract ABI, for contract deployment.
 * Extends the ContractFunctionInteraction class.
 */
export class DeployMethod<TContract extends ContractBase = Contract> extends ContractFunctionInteraction {
  /**
   * The partially computed contract address. Known after creation of the deployment transaction.
   */
  public partialContractAddress?: PartialContractAddress = undefined;

  /**
   * The complete contract address.
   */
  public completeContractAddress?: AztecAddress = undefined;

  constructor(private publicKey: PublicKey, private arc: AztecRPC, private abi: ContractAbi, args: any[] = []) {
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    super(new DeployerWallet(arc), AztecAddress.ZERO, constructorAbi, args);
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

    const { chainId, version } = await this.wallet.getNodeInfo();

    const { address, constructorHash, functionTreeRoot, partialAddress } = await getContractDeploymentInfo(
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
    const executionRequest = await this.getExecutionRequest(address, AztecAddress.ZERO);
    const txRequest = await this.wallet.createAuthenticatedTxRequest([executionRequest], txContext);

    this.txRequest = txRequest;
    this.partialContractAddress = partialAddress;
    this.completeContractAddress = address;

    // TODO: Should we add the contracts to the DB here, or once the tx has been sent or mined?
    await this.wallet.addContracts([{ abi: this.abi, address, portalContract }]);

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
  public async simulate(options: DeployOptions): Promise<Tx> {
    const txRequest = this.txRequest ?? (await this.create(options));

    this.tx = await this.wallet.simulateTx(txRequest);
    return this.tx;
  }

  /**
   * Creates a contract abstraction given a wallet.
   * @param withWallet - The wallet to provide to the contract abstraction
   * @returns - The generated contract abstraction.
   */
  public async getContract(withWallet: Wallet) {
    if (!this.completeContractAddress) {
      throw new Error(`Cannot get a contract instance for a contract not yet deployed`);
    }
    return await Contract.create(this.completeContractAddress, this.abi, withWallet);
  }
}
