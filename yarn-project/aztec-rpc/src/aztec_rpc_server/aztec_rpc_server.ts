import { encodeArguments } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import {
  AztecAddress,
  ContractDeploymentData,
  EcdsaSignature,
  EthAddress,
  FunctionData,
  TxContext,
} from '@aztec/circuits.js';
import { ContractAbi, FunctionType } from '@aztec/foundation/abi';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { KeyStore, PublicKey, getAddressFromPublicKey } from '@aztec/key-store';
import { AccountContractAbi } from '@aztec/noir-contracts/examples';
import { ExecutionRequest, SignedTxExecutionRequest, Tx, TxExecutionRequest, TxHash } from '@aztec/types';
import { EcdsaAccountContract } from '../account_impl/ecdsa_account_contract.js';
import { EcdsaExternallyOwnedAccount } from '../account_impl/ecdsa_eoa.js';
import { AccountImplementation } from '../account_impl/index.js';
import { AccountState } from '../account_state/account_state.js';
import { AztecRPCClient, DeployedContract } from '../aztec_rpc_client/index.js';
import { ContractDao, toContractDao } from '../contract_database/index.js';
import { ContractTree } from '../contract_tree/index.js';
import { Database, TxDao } from '../database/index.js';
import { Synchroniser } from '../synchroniser/index.js';
import { TxReceipt, TxStatus } from '../tx/index.js';

/**
 * A remote Aztec RPC Client implementation.
 */
export class AztecRPCServer implements AztecRPCClient {
  private synchroniser: Synchroniser;

  constructor(
    private keyStore: KeyStore,
    private node: AztecNode,
    private db: Database,
    private log = createDebugLogger('aztec:rpc_server'),
  ) {
    this.synchroniser = new Synchroniser(node, db);
  }

  /**
   * Starts the Aztec RPC server by initializing account states for each registered account and
   * begins the synchronisation process between the Aztec node and the database.
   * It logs the number of initial accounts that were started.
   *
   * @returns A promise that resolves when the server has started successfully.
   */
  public async start() {
    const accounts = await this.keyStore.getAccounts();
    for (const account of accounts) {
      await this.initAccountState(account, getAddressFromPublicKey(account));
    }
    await this.synchroniser.start();
    this.log(`Started. ${accounts.length} initial accounts.`);
  }

  /**
   * Stops the Aztec RPC server, halting processing of new transactions and shutting down the synchronizer.
   * This function ensures that all ongoing tasks are completed before stopping the server.
   * It is useful for gracefully shutting down the server during maintenance or restarts.
   *
   * @returns A Promise resolving once the server has been stopped successfully.
   */
  public async stop() {
    await this.synchroniser.stop();
    this.log('Stopped.');
  }

  /**
   * Adds a new account to the AztecRPCServer instance.
   *
   * @returns The AztecAddress of the newly added account.
   * @deprecated EOAs to be removed.
   */
  public async addExternallyOwnedAccount() {
    const accountPubKey = await this.keyStore.createAccount();
    const address = getAddressFromPublicKey(accountPubKey);
    await this.initAccountState(accountPubKey, address);
    return address;
  }

  /**
   * Creates or registers a new keypair in the keystore and deploys a new account contract for it.
   * @param privKey - Private key to use for the deployment (a fresh one will be generated if not set).
   * @returns A tuple with the deployment tx to be awaited and the address of the account.
   */
  public async createSmartAccount(privKey?: Buffer): Promise<[TxHash, AztecAddress]> {
    const pubKey = await (privKey ? this.keyStore.addAccount(privKey) : this.keyStore.createAccount());
    const portalContract = EthAddress.ZERO;
    const contractAddressSalt = Fr.random();
    const abi = AccountContractAbi;
    const args: any[] = [];

    const { txRequest, contract } = await this.prepareDeploy(abi, args, portalContract, contractAddressSalt, pubKey);

    const account = await this.initAccountState(pubKey, contract.address);

    const tx = await account.simulateAndProve(
      new SignedTxExecutionRequest(txRequest, EcdsaSignature.empty()),
      contract.address,
    );

    await this.db.addTx(
      new TxDao(await tx.getTxHash(), undefined, undefined, account.getAddress(), undefined, contract.address, ''),
    );

    return [await this.sendTx(tx), account.getAddress()];
  }

  /**
   * Registers an account backed by an account contract.
   *
   * TODO: We should not be passing in the private key in plain, instead, we should ask the keystore for a public key, create the smart account with it, and register it here.
   * @param privKey - Private key of the corresponding user master public key.
   * @param address - Address of the account contract.
   * @returns The address of the account contract.
   */
  public async registerSmartAccount(privKey: Buffer, address: AztecAddress) {
    const pubKey = await this.keyStore.addAccount(privKey);
    await this.initAccountState(pubKey, address);
    return address;
  }

  /**
   * Add an array of deployed contracts to the database.
   * Each contract should contain ABI, address, and portalContract information.
   *
   * @param contracts - An array of DeployedContract objects containing contract ABI, address, and portalContract.
   * @returns A Promise that resolves once all the contracts have been added to the database.
   */
  public async addContracts(contracts: DeployedContract[]) {
    const contractDaos = contracts.map(c => toContractDao(c.abi, c.address, c.portalContract));
    await Promise.all(contractDaos.map(c => this.db.addContract(c)));
  }

  /**
   * Retrieves the list of Aztec addresses associated with the current accounts in the key store.
   * The addresses are returned as a promise that resolves to an array of AztecAddress objects.
   *
   * @returns A promise that resolves to an array of AztecAddress instances.
   */
  public async getAccounts(): Promise<AztecAddress[]> {
    const accounts = this.synchroniser.getAccounts();
    return await Promise.all(accounts.map(a => a.getAddress()));
  }

  /**
   * Retrieve the public key associated with an address.
   * Throws an error if the account is not found in the key store.
   *
   * @param address - The AztecAddress instance representing the account.
   * @returns A Promise resolving to the Point instance representing the public key.
   */
  public getAccountPublicKey(address: AztecAddress): Promise<Point> {
    const account = this.ensureAccount(address);
    return Promise.resolve(account.getPublicKey());
  }

  /**
   * Retrieves the storage data at a specified contract address and storage slot.
   * The returned data is an array of note preimage items, with each item containing its value.
   *
   * @param contract - The AztecAddress of the target contract.
   * @param storageSlot - The Fr representing the storage slot to be fetched.
   * @returns A promise that resolves to an array of note preimage items, each containing its value.
   */
  public async getStorageAt(contract: AztecAddress, storageSlot: Fr) {
    const noteSpendingInfo = await this.db.getNoteSpendingInfo(contract, storageSlot);
    return noteSpendingInfo.map(d => d.notePreimage.items.map(item => item.value));
  }

  /**
   * Is an L2 contract deployed at this address?
   * @param contractAddress - The contract data address.
   * @returns Whether the contract was deployed.
   */
  public async isContractDeployed(contractAddress: AztecAddress): Promise<boolean> {
    return !!(await this.node.getContractInfo(contractAddress));
  }

  /**
   * Create a deployment transaction request for deploying a new contract.
   * The function generates ContractDeploymentData and a TxRequest instance containing
   * the constructor function data, flat arguments, nonce, and other necessary information.
   * This TxRequest can then be signed and sent to deploy the contract on the Aztec network.
   *
   * @param abi - The contract ABI containing function definitions.
   * @param args - The arguments required for the constructor function of the contract.
   * @param portalContract - The Ethereum address of the portal contract.
   * @param contractAddressSalt - (Optional) Salt value used to generate the contract address.
   * @param from - (Optional) The Aztec address of the account that deploys the contract.
   * @returns A TxRequest instance containing all necessary information for contract deployment.
   */
  public async createDeploymentTx(
    abi: ContractAbi,
    args: any[],
    portalContract: EthAddress,
    contractAddressSalt = Fr.random(),
    from?: AztecAddress,
  ) {
    const account = this.ensureAccountOrDefault(from);
    const pubKey = account.getPublicKey();

    const { txRequest, contract } = await this.prepareDeploy(abi, args, portalContract, contractAddressSalt, pubKey);

    const tx = await account.simulateAndProve(
      new SignedTxExecutionRequest(txRequest, EcdsaSignature.empty()),
      contract.address,
    );

    await this.db.addTx(
      new TxDao(await tx.getTxHash(), undefined, undefined, account.getAddress(), undefined, contract.address, ''),
    );

    return tx;
  }

  private async prepareDeploy(
    abi: ContractAbi,
    args: any[],
    portalContract: EthAddress,
    contractAddressSalt: Fr,
    pubKey: PublicKey,
  ) {
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    const flatArgs = encodeArguments(constructorAbi, args);
    const contractTree = await ContractTree.new(abi, flatArgs, portalContract, contractAddressSalt, pubKey, this.node);
    const { functionData, vkHash } = contractTree.newContractConstructor!;
    const functionTreeRoot = await contractTree.getFunctionTreeRoot();
    const contractDeploymentData = new ContractDeploymentData(
      pubKey,
      Fr.fromBuffer(vkHash),
      functionTreeRoot,
      contractAddressSalt,
      portalContract,
    );
    const txContext = new TxContext(false, false, true, contractDeploymentData);

    const contract = contractTree.contract;
    await this.db.addContract(contract);

    const txRequest = new TxExecutionRequest(
      AztecAddress.ZERO,
      contract.address,
      functionData,
      flatArgs,
      Fr.random(),
      txContext,
      Fr.ZERO,
    );
    return { txRequest, contract };
  }

  /**
   * Create a transaction for a contract function call with the provided arguments.
   * Throws an error if the contract or function is unknown.
   *
   * @param functionName - Name of the function to be invoked in the contract.
   * @param args - Array of input arguments for the function.
   * @param to - Address of the target contract.
   * @param optionalFromAddress - (Optional) Address of the sender (defaults to first available account).
   * @returns A Tx ready to send to the p2p pool for execution.
   */
  public async createTx(functionName: string, args: any[], to: AztecAddress, optionalFromAddress?: AztecAddress) {
    const account = this.ensureAccountOrDefault(optionalFromAddress);
    const accountContract = await this.db.getContract(account.getAddress());
    const entrypoint: AccountImplementation = this.getAccountImplementation(account, accountContract);

    const executionRequest = await this.getExecutionRequest(account, functionName, args, to);

    // TODO: Can we remove tx context from this call?
    const authedTxRequest = await entrypoint.createAuthenticatedTxRequest([executionRequest], TxContext.empty());
    const tx = authedTxRequest.txRequest.functionData.isPrivate
      ? await account.simulateAndProve(authedTxRequest, undefined)
      : Tx.createPublic(authedTxRequest);

    await this.db.addTx(new TxDao(await tx.getTxHash(), undefined, undefined, account.getAddress(), to, undefined, ''));

    return tx;
  }

  private async getExecutionRequest(
    account: AccountState,
    functionName: string,
    args: any[],
    to: AztecAddress,
  ): Promise<ExecutionRequest> {
    const contract = await this.db.getContract(to);
    if (!contract) {
      throw new Error('Unknown contract.');
    }

    const functionDao = contract.functions.find(f => f.name === functionName);
    if (!functionDao) {
      throw new Error('Unknown function.');
    }

    const flatArgs = encodeArguments(functionDao, args);

    const functionData = new FunctionData(
      functionDao.selector,
      functionDao.functionType === FunctionType.SECRET,
      false,
    );

    return {
      args: flatArgs,
      from: account.getAddress(),
      functionData,
      to,
    };
  }

  // TODO: Store the kind of account in account state
  private getAccountImplementation(accountState: AccountState, contract: ContractDao | undefined) {
    const address = accountState.getAddress();
    const pubKey = accountState.getPublicKey();

    if (!contract) {
      this.log(`Using ECDSA EOA implementation for ${address}`);
      return new EcdsaExternallyOwnedAccount(address, pubKey, this.keyStore);
    } else if (contract.name === 'Account') {
      this.log(`Using ECDSA account contract implementation for ${address}`);
      return new EcdsaAccountContract(address, pubKey, this.keyStore);
    } else {
      throw new Error(`Unknown account implementation for ${address}`);
    }
  }

  /**
   * Send a transaction.
   * @param tx - The transaction.
   * @returns A hash of the transaction, used to identify it.
   */
  public async sendTx(tx: Tx): Promise<TxHash> {
    await this.node.sendTx(tx);
    return tx.getTxHash();
  }

  /**
   * Simulate the execution of a view (read-only) function on a deployed contract without actually modifying state.
   * This is useful to inspect contract state, for example fetching a variable value or calling a getter function.
   * The function takes function name and arguments as parameters, along with the contract address
   * and optionally the sender's address.
   *
   * @param functionName - The name of the function to be called in the contract.
   * @param args - An array of arguments to be passed into the function call.
   * @param to - The address of the contract on which the function will be called.
   * @param from - (Optional) The caller of the transaction.
   * @returns The result of the view function call, structured based on the function ABI.
   */
  public async viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress) {
    const account = this.ensureAccountOrDefault(from);
    const txRequest = await this.getExecutionRequest(account, functionName, args, to);

    const executionResult = await account.simulateUnconstrained(txRequest);

    // TODO - Return typed result based on the function abi.
    return executionResult;
  }

  /**
   * Fetchs a transaction receipt for a tx.
   * @param txHash - The transaction hash.
   * @returns A recipt of the transaction.
   */
  public async getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    const localTx = await this.synchroniser.getTxByHash(txHash);
    const partialReceipt = {
      txHash: txHash,
      blockHash: localTx?.blockHash,
      blockNumber: localTx?.blockNumber,
      from: localTx?.from,
      to: localTx?.to,
      contractAddress: localTx?.contractAddress,
      error: '',
    };

    if (localTx?.blockHash) {
      return {
        ...partialReceipt,
        status: TxStatus.MINED,
      };
    }

    const pendingTx = await this.node.getPendingTxByHash(txHash);
    if (pendingTx) {
      return {
        ...partialReceipt,
        status: TxStatus.PENDING,
      };
    }

    // if the transaction mined it will be removed from the pending pool and there is a race condition here as the synchroniser will not have the tx as mined yet, so it will appear dropped
    // until the synchroniser picks this up

    const accountState = this.synchroniser.getAccount(localTx.from);
    if (accountState && !(await accountState?.isSynchronised())) {
      // there is a pending L2 block, which means the transaction will not be in the tx pool but may be awaiting mine on L1
      return {
        ...partialReceipt,
        status: TxStatus.PENDING,
      };
    }

    // TODO we should refactor this once the node can store transactions. At that point we should query the node and not deal with block heights.

    return {
      ...partialReceipt,
      status: TxStatus.DROPPED,
      error: 'Tx dropped by P2P node.',
    };
  }

  /**
   * Initializes the account state for a given address.
   * It retrieves the private key from the key store and adds the account to the synchroniser.
   * This function is called for all existing accounts during the server start, or when a new account is added afterwards.
   *
   * @param pubKey - User's master public key.
   * @param address - The address of the account to initialize.
   */
  private async initAccountState(pubKey: PublicKey, address: AztecAddress) {
    const accountPrivateKey = await this.keyStore.getAccountPrivateKey(pubKey);
    const account = await this.synchroniser.addAccount(accountPrivateKey, address);
    this.log(`Account added: ${address.toString()}`);
    return account;
  }

  /**
   * Retrieves an existing account or the default one if none is provided.
   * Ensures that the given account address exists in the synchroniser, otherwise throws an error.
   * If no account address is provided, it returns the first account from the synchroniser.
   * Throws an error if there are no accounts available in the key store.
   *
   * @param account - (Optional) Address of the account to ensure its existence.
   * @returns The ensured account instance.
   */
  private ensureAccountOrDefault(account?: AztecAddress) {
    const address = account || this.synchroniser.getAccounts()[0]?.getAddress();
    if (!address) {
      throw new Error('No accounts available in the key store.');
    }

    return this.ensureAccount(address);
  }

  /**
   * Ensures the given account address exists in the synchroniser.
   * Retrieves the account state for the provided address and throws an error if the account is not found.
   *
   * @param account - The account address.
   * @returns The account state associated with the given address.
   * @throws If the account is unknown or not found in the synchroniser.
   */
  private ensureAccount(account: AztecAddress) {
    const accountState = this.synchroniser.getAccount(account);
    if (!accountState) {
      throw new Error(`Unknown account: ${account.toShortString()}.`);
    }

    return accountState;
  }
}
