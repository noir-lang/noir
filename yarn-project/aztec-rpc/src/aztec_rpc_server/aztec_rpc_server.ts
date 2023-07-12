import { encodeArguments } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { AztecAddress, FunctionData } from '@aztec/circuits.js';
import { FunctionType } from '@aztec/foundation/abi';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { KeyStore, PublicKey } from '@aztec/key-store';
import { SchnorrAccountContractAbi } from '@aztec/noir-contracts/examples';
import {
  ContractData,
  ContractPublicData,
  ExecutionRequest,
  L2BlockL2Logs,
  LogType,
  PartialContractAddress,
  Tx,
  TxExecutionRequest,
  TxHash,
} from '@aztec/types';
import { AccountState } from '../account_state/account_state.js';
import { AztecRPC, DeployedContract, NodeInfo } from '../aztec_rpc/index.js';
import { toContractDao } from '../contract_database/index.js';
import { Database, TxDao } from '../database/index.js';
import { Synchroniser } from '../synchroniser/index.js';
import { TxReceipt, TxStatus } from '../tx/index.js';

/**
 * A remote Aztec RPC Client implementation.
 */
export class AztecRPCServer implements AztecRPC {
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
    await this.synchroniser.start();
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
   * Registers an account backed by an account contract.
   *
   * TODO: We should not be passing in the private key in plain, instead, we should ask the keystore for a public key, create the smart account with it, and register it here.
   * @param privKey - Private key of the corresponding user master public key.
   * @param address - Address of the account contract.
   * @param partialContractAddress - The partially computed address of the account contract.
   * @param abi - Implementation of the account contract backed by this account.
   * @returns The address of the account contract.
   */
  public async addAccount(
    privKey: Buffer,
    address: AztecAddress,
    partialContractAddress: PartialContractAddress,
    abi = SchnorrAccountContractAbi,
  ) {
    const pubKey = this.keyStore.addAccount(privKey);
    // TODO(#1007): ECDSA contract breaks this check, since the ecdsa public key does not match the one derived from the keystore.
    // Once we decouple the ecdsa contract signing and encryption keys, we can re-enable this check.
    // const wasm = await CircuitsWasm.get();
    // const expectedAddress = computeContractAddressFromPartial(wasm, pubKey, partialContractAddress);
    // if (!expectedAddress.equals(address)) {
    //   throw new Error(
    //     `Address cannot be derived from pubkey and partial address (received ${address.toString()}, derived ${expectedAddress.toString()})`,
    //   );
    // }
    await this.db.addPublicKey(address, pubKey, partialContractAddress);
    await this.#initAccountState(pubKey, address, partialContractAddress, abi);
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
    for (const contract of contractDaos) {
      this.log(
        `Added contract ${contract.name} at ${contract.address} with portal ${contract.portalContract} to the local db`,
      );
    }
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
    const account = this.#ensureAccountAddress(address);
    return Promise.resolve(account.getPublicKey());
  }

  /**
   * Retrieve the address associated with a public key.
   * Throws an error if the account is not found in the key store.
   *
   * @param publicKey - The Point instance representing the account public key.
   * @returns A Promise resolving to the Aztec Address.
   */
  public getAccountAddress(publicKey: Point): Promise<AztecAddress> {
    // const account = this.#ensureAccount(address);
    const account = this.#ensureAccountPublicKey(publicKey);
    return Promise.resolve(account.getAddress());
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
   * Create a transaction for a contract function call with the provided arguments.
   * Throws an error if the contract or function is unknown.
   *
   * @param txRequest - An authenticated tx request ready for simulation
   * @param optionalFromAddress - The address to simulate from
   * @returns A Tx ready to send to the p2p pool for execution.
   */
  public async simulateTx(txRequest: TxExecutionRequest, optionalFromAddress?: AztecAddress) {
    const account = this.#ensureAccountOrDefault(optionalFromAddress);

    if (!txRequest.functionData.isPrivate) {
      throw new Error(`Public entrypoints are not allowed`);
    }

    // We get the contract address from origin, since contract deployments are signalled as origin from their own address
    // TODO: Is this ok? Should it be changed to be from ZERO?
    const deployedContractAddress = txRequest.txContext.isContractDeploymentTx ? txRequest.origin : undefined;
    const newContract = deployedContractAddress ? await this.db.getContract(deployedContractAddress) : undefined;

    const tx = await account.simulateAndProve(txRequest, newContract);

    await this.db.addTx(
      TxDao.from({
        txHash: await tx.getTxHash(),
        from: account.getAddress(),
        to: account.getAddress(),
        contractAddress: deployedContractAddress,
      }),
    );

    return tx;
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
   * @param args - The arguments to be provided to the function.
   * @param to - The address of the contract to be called.
   * @param from - (Optional) The caller of the transaction.
   * @returns The result of the view function call, structured based on the function ABI.
   */
  public async viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress) {
    const account = this.#ensureAccountOrDefault(from);
    const txRequest = await this.#getExecutionRequest(account, functionName, args, to);

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
   * Get latest L2 block number.
   * @returns The latest block number.
   */
  async getBlockNum(): Promise<number> {
    return await this.node.getBlockHeight();
  }

  /**
   * Lookup the L2 contract data for this contract.
   * Contains the ethereum portal address and bytecode.
   * @param contractAddress - The contract data address.
   * @returns The complete contract data including portal address & bytecode (if we didn't throw an error).
   */
  public async getContractData(contractAddress: AztecAddress): Promise<ContractPublicData | undefined> {
    return await this.node.getContractData(contractAddress);
  }

  /**
   * Lookup the L2 contract info for this contract.
   * Contains the ethereum portal address .
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  public async getContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return await this.node.getContractInfo(contractAddress);
  }

  /**
   * Gets L2 block unencrypted logs.
   * @param from - Number of the L2 block to which corresponds the first unencrypted logs to be returned.
   * @param take - The number of unencrypted logs to return.
   * @returns The requested unencrypted logs.
   */
  public async getUnencryptedLogs(from: number, take: number): Promise<L2BlockL2Logs[]> {
    return await this.node.getLogs(from, take, LogType.UNENCRYPTED);
  }

  /**
   * Initializes the account state for a given address.
   * It retrieves the private key from the key store and adds the account to the synchroniser.
   * This function is called for all existing accounts during the server start, or when a new account is added afterwards.
   *
   * @param pubKey - User's master public key.
   * @param address - The address of the account to initialize.
   * @param partialContractAddress - The partially computed account contract address.
   * @param curve - The curve to be used for elliptic curve operations.
   * @param signer - The signer to be used for transaction signing.
   * @param abi - Implementation of the account contract backing the account.
   */
  async #initAccountState(
    pubKey: PublicKey,
    address: AztecAddress,
    partialContractAddress: PartialContractAddress,
    abi = SchnorrAccountContractAbi,
  ) {
    const account = await this.synchroniser.addAccount(pubKey, address, partialContractAddress, abi, this.keyStore);
    this.log(`Added account ${address.toString()} with pubkey ${pubKey.toString()}`);
    return account;
  }

  async #getExecutionRequest(
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

  /**
   * Retrieves an existing account or the default one if none is provided.
   * Ensures that the given account address exists in the synchroniser, otherwise throws an error.
   * If no account address is provided, it returns the first account from the synchroniser.
   * Throws an error if there are no accounts available in the key store.
   *
   * @param account - (Optional) Address of the account to ensure its existence.
   * @returns The ensured account instance.
   */
  #ensureAccountOrDefault(account?: AztecAddress) {
    const address = account || this.synchroniser.getAccounts()[0]?.getAddress();
    if (!address) {
      throw new Error('No accounts available in the key store.');
    }

    return this.#ensureAccountAddress(address);
  }

  /**
   * Ensures the given account address exists in the synchroniser.
   * Retrieves the account state for the provided address and throws an error if the account is not found.
   *
   * @param account - The account address.
   * @returns The account state associated with the given address.
   * @throws If the account is unknown or not found in the synchroniser.
   */
  #ensureAccountAddress(account: AztecAddress) {
    const accountState = this.synchroniser.getAccount(account);
    if (!accountState) {
      throw new Error(`Unknown account: ${account.toShortString()}.`);
    }

    return accountState;
  }

  /**
   * Ensures the given account public key exists in the synchroniser.
   * Retrieves the account state for the provided address and throws an error if the account is not found.
   *
   * @param account - The public key.
   * @returns The account state associated with the given address.
   * @throws If the account is unknown or not found in the synchroniser.
   */
  #ensureAccountPublicKey(account: Point) {
    const accountState = this.synchroniser.getAccountByPublicKey(account);

    if (!accountState) {
      throw new Error(`Unknown account: ${account.toShortString()}.`);
    }

    return accountState;
  }

  /**
   * Returns the information about the server's node
   * @returns - The node information.
   */
  public async getNodeInfo(): Promise<NodeInfo> {
    const [version, chainId] = await Promise.all([this.node.getVersion(), this.node.getChainId()]);

    return {
      version,
      chainId,
    };
  }
}
