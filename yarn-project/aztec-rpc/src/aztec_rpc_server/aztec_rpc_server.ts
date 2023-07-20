import {
  collectEncryptedLogs,
  collectEnqueuedPublicFunctionCalls,
  collectUnencryptedLogs,
} from '@aztec/acir-simulator';
import {
  AztecAddress,
  FunctionData,
  PartialContractAddress,
  PrivateHistoricTreeRoots,
  PublicKey,
} from '@aztec/circuits.js';
import { FunctionType, encodeArguments } from '@aztec/foundation/abi';
import { Fr, Point } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import {
  AztecNode,
  AztecRPC,
  ContractDao,
  ContractData,
  ContractPublicData,
  DeployedContract,
  ExecutionRequest,
  KeyStore,
  L2BlockL2Logs,
  LogType,
  MerkleTreeId,
  NodeInfo,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxL2Logs,
  TxReceipt,
  TxStatus,
  getNewContractPublicFunctions,
  toContractDao,
} from '@aztec/types';

import { RpcServerConfig } from '../config/index.js';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { Database, TxDao } from '../database/index.js';
import { KernelOracle } from '../kernel_oracle/index.js';
import { KernelProver } from '../kernel_prover/kernel_prover.js';
import { getAcirSimulator } from '../simulator/index.js';
import { Synchroniser } from '../synchroniser/index.js';

/**
 * A remote Aztec RPC Client implementation.
 */
export class AztecRPCServer implements AztecRPC {
  private synchroniser: Synchroniser;
  private log: DebugLogger;

  constructor(
    private keyStore: KeyStore,
    private node: AztecNode,
    private db: Database,
    private config: RpcServerConfig,
    logSuffix = '0',
  ) {
    this.log = createDebugLogger('aztec:rpc_server_' + logSuffix);
    this.synchroniser = new Synchroniser(node, db, logSuffix);
  }

  /**
   * Starts the Aztec RPC server by beginning the synchronisation process between the Aztec node and the database.
   *
   * @returns A promise that resolves when the server has started successfully.
   */
  public async start() {
    await this.synchroniser.start(1, 1, this.config.l2BlockPollingIntervalMS);
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
   * @param privKey - Private key of the corresponding user master public key.
   * @param address - Address of the account contract.
   * @param partialContractAddress - The partially computed address of the account contract.
   * @returns The address of the account contract.
   */
  public async addAccount(privKey: Buffer, address: AztecAddress, partialContractAddress: PartialContractAddress) {
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
    await this.db.addPublicKeyAndPartialAddress(address, pubKey, partialContractAddress);
    this.synchroniser.addAccount(pubKey, address, this.keyStore);
    return address;
  }

  /**
   * Adds public key and partial address to a database.
   * @param address - Address of the account to add public key and partial address for.
   * @param publicKey - Public key of the corresponding user.
   * @param partialAddress - The partially computed address of the account contract.
   * @returns A Promise that resolves once the public key has been added to the database.
   */
  public async addPublicKeyAndPartialAddress(
    address: AztecAddress,
    publicKey: PublicKey,
    partialAddress: PartialContractAddress,
  ): Promise<void> {
    await this.db.addPublicKeyAndPartialAddress(address, publicKey, partialAddress);
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
   * Retrieves the list of Aztec addresses added to this rpc server
   * The addresses are returned as a promise that resolves to an array of AztecAddress objects.
   *
   * @returns A promise that resolves to an array of AztecAddress instances.
   */
  public async getAccounts(): Promise<AztecAddress[]> {
    return await this.db.getAccounts();
  }

  /**
   * Retrieve the public key associated with an address.
   * Throws an error if the account is not found in the key store.
   *
   * @param address - The AztecAddress instance representing the account to get public key for.
   * @returns A Promise resolving to the Point instance representing the public key.
   */
  public async getPublicKey(address: AztecAddress): Promise<Point> {
    const result = await this.db.getPublicKeyAndPartialAddress(address);
    if (!result) {
      throw new Error(`Unable to retrieve public key for address ${address.toString()}`);
    }
    return Promise.resolve(result[0]);
  }

  /**
   * Retrieve the public key and partial contract address associated with an address.
   * Throws an error if the account is not found in the key store.
   *
   * @param address - The AztecAddress instance representing the account to get public key and partial address for.
   * @returns A Promise resolving to the Point instance representing the public key.
   */
  public async getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[Point, PartialContractAddress]> {
    const result = await this.db.getPublicKeyAndPartialAddress(address);
    if (!result) {
      throw new Error(`Unable to get public key for address ${address.toString()}`);
    }
    return Promise.resolve(result);
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
   * Retrieves the public storage data at a specified contract address and storage slot.
   * The returned data is an array of note preimage items, with each item containing its value.
   *
   * @param contract - The AztecAddress of the target contract.
   * @param storageSlot - The Fr representing the storage slot to be fetched.
   * @returns A promise that resolves to an array of note preimage items, each containing its value.
   */
  public async getPublicStorageAt(contract: AztecAddress, storageSlot: Fr) {
    return await this.node.getStorageAt(contract, storageSlot.value);
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
  public async simulateTx(txRequest: TxExecutionRequest) {
    if (!txRequest.functionData.isPrivate) {
      throw new Error(`Public entrypoints are not allowed`);
    }

    // We get the contract address from origin, since contract deployments are signalled as origin from their own address
    // TODO: Is this ok? Should it be changed to be from ZERO?
    const deployedContractAddress = txRequest.txContext.isContractDeploymentTx ? txRequest.origin : undefined;
    const newContract = deployedContractAddress ? await this.db.getContract(deployedContractAddress) : undefined;

    const tx = await this.#simulateAndProve(txRequest, newContract);

    await this.db.addTx(
      TxDao.from({
        txHash: await tx.getTxHash(),
        origin: txRequest.origin,
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
    const txRequest = await this.#getExecutionRequest(functionName, args, to, from ?? AztecAddress.ZERO);

    const executionResult = await this.#simulateUnconstrained(txRequest);

    // TODO - Return typed result based on the function abi.
    return executionResult;
  }

  /**
   * Fetches a transaction receipt for a tx.
   * @param txHash - The transaction hash.
   * @returns A receipt of the transaction.
   */
  public async getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    const localTx = await this.#getTxByHash(txHash);
    const partialReceipt = new TxReceipt(
      txHash,
      TxStatus.PENDING,
      '',
      localTx?.blockHash,
      localTx?.blockNumber,
      localTx?.origin,
      localTx?.contractAddress,
    );

    if (localTx?.blockHash) {
      partialReceipt.status = TxStatus.MINED;
      return partialReceipt;
    }

    const pendingTx = await this.node.getPendingTxByHash(txHash);
    if (pendingTx) {
      return partialReceipt;
    }

    // if the transaction mined it will be removed from the pending pool and there is a race condition here as the synchroniser will not have the tx as mined yet, so it will appear dropped
    // until the synchroniser picks this up

    const isSynchronised = await this.synchroniser.isSynchronised();
    if (!isSynchronised) {
      // there is a pending L2 block, which means the transaction will not be in the tx pool but may be awaiting mine on L1
      return partialReceipt;
    }

    // TODO we should refactor this once the node can store transactions. At that point we should query the node and not deal with block heights.
    partialReceipt.status = TxStatus.DROPPED;
    partialReceipt.error = 'Tx dropped by P2P node.';
    return partialReceipt;
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

  async #getExecutionRequest(
    functionName: string,
    args: any[],
    to: AztecAddress,
    from: AztecAddress,
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
      from,
      functionData,
      to,
    };
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

  /**
   * Retrieve a transaction by its hash from the database.
   *
   * @param txHash - The hash of the transaction to be fetched.
   * @returns A TxDao instance representing the retrieved transaction.
   */
  async #getTxByHash(txHash: TxHash): Promise<TxDao> {
    const tx = await this.db.getTx(txHash);
    if (!tx) {
      throw new Error(`Transaction ${txHash} not found in RPC database`);
    }
    return tx;
  }

  /**
   * Retrieves the simulation parameters required to run an ACIR simulation.
   * This includes the contract address, function ABI, portal contract address, and historic tree roots.
   * The function uses the given 'contractDataOracle' to fetch the necessary data from the node and user's database.
   *
   * @param execRequest - The transaction request object containing details of the contract call.
   * @param contractDataOracle - An instance of ContractDataOracle used to fetch the necessary data.
   * @returns An object containing the contract address, function ABI, portal contract address, and historic tree roots.
   */
  async #getSimulationParameters(
    execRequest: ExecutionRequest | TxExecutionRequest,
    contractDataOracle: ContractDataOracle,
  ) {
    const contractAddress = (execRequest as ExecutionRequest).to ?? (execRequest as TxExecutionRequest).origin;
    const functionAbi = await contractDataOracle.getFunctionAbi(
      contractAddress,
      execRequest.functionData.functionSelectorBuffer,
    );
    const portalContract = await contractDataOracle.getPortalContractAddress(contractAddress);

    const currentRoots = this.db.getTreeRoots();
    const historicRoots = PrivateHistoricTreeRoots.from({
      contractTreeRoot: currentRoots[MerkleTreeId.CONTRACT_TREE],
      nullifierTreeRoot: currentRoots[MerkleTreeId.NULLIFIER_TREE],
      privateDataTreeRoot: currentRoots[MerkleTreeId.PRIVATE_DATA_TREE],
      l1ToL2MessagesTreeRoot: currentRoots[MerkleTreeId.L1_TO_L2_MESSAGES_TREE],
      privateKernelVkTreeRoot: Fr.ZERO,
    });

    return {
      contractAddress,
      functionAbi,
      portalContract,
      historicRoots,
    };
  }

  /**
   * Simulate the execution of a transaction request.
   * This function computes the expected state changes resulting from the transaction
   * without actually submitting it to the blockchain. The result will be used for creating the kernel proofs,
   * as well as for estimating gas costs.
   *
   * @param txRequest - The transaction request object containing the necessary data for simulation.
   * @param contractDataOracle - Optional parameter, an instance of ContractDataOracle class for retrieving contract data.
   * @returns A promise that resolves to an object containing the simulation results, including expected output notes and any error messages.
   */
  async #simulate(txRequest: TxExecutionRequest, contractDataOracle?: ContractDataOracle) {
    // TODO - Pause syncing while simulating.
    if (!contractDataOracle) {
      contractDataOracle = new ContractDataOracle(this.db, this.node);
    }

    const { contractAddress, functionAbi, portalContract, historicRoots } = await this.#getSimulationParameters(
      txRequest,
      contractDataOracle,
    );

    const simulator = getAcirSimulator(this.db, this.node, this.node, this.node, this.keyStore, contractDataOracle);

    try {
      this.log('Executing simulator...');
      const result = await simulator.run(txRequest, functionAbi, contractAddress, portalContract, historicRoots);
      this.log('Simulation completed!');

      return result;
    } catch (err: any) {
      throw typeof err === 'string' ? new Error(err) : err; // Work around raw string being thrown
    }
  }

  /**
   * Simulate an unconstrained transaction on the given contract, without considering constraints set by ACIR.
   * The simulation parameters are fetched using ContractDataOracle and executed using AcirSimulator.
   * Returns the simulation result containing the outputs of the unconstrained function.
   *
   * @param execRequest - The transaction request object containing the target contract and function data.
   * @param contractDataOracle - Optional instance of ContractDataOracle for fetching and caching contract information.
   * @returns The simulation result containing the outputs of the unconstrained function.
   */
  async #simulateUnconstrained(execRequest: ExecutionRequest, contractDataOracle?: ContractDataOracle) {
    if (!contractDataOracle) {
      contractDataOracle = new ContractDataOracle(this.db, this.node);
    }

    const { contractAddress, functionAbi, portalContract, historicRoots } = await this.#getSimulationParameters(
      execRequest,
      contractDataOracle,
    );

    const simulator = getAcirSimulator(this.db, this.node, this.node, this.node, this.keyStore, contractDataOracle);

    this.log('Executing unconstrained simulator...');
    const result = await simulator.runUnconstrained(
      execRequest,
      functionAbi,
      contractAddress,
      portalContract,
      historicRoots,
    );
    this.log('Unconstrained simulation completed!');

    return result;
  }

  /**
   * Simulate a transaction, generate a kernel proof, and create a private transaction object.
   * The function takes in a transaction request and an ECDSA signature. It simulates the transaction,
   * then generates a kernel proof using the simulation result. Finally, it creates a private
   * transaction object with the generated proof and public inputs. If a new contract address is provided,
   * the function will also include the new contract's public functions in the transaction object.
   *
   * @param txExecutionRequest - The transaction request to be simulated and proved.
   * @param signature - The ECDSA signature for the transaction request.
   * @param newContract - Optional. The address of a new contract to be included in the transaction object.
   * @returns A private transaction object containing the proof, public inputs, and encrypted logs.
   */
  async #simulateAndProve(txExecutionRequest: TxExecutionRequest, newContract: ContractDao | undefined) {
    // TODO - Pause syncing while simulating.

    const contractDataOracle = new ContractDataOracle(this.db, this.node);

    const kernelOracle = new KernelOracle(contractDataOracle, this.node);
    const executionResult = await this.#simulate(txExecutionRequest, contractDataOracle);

    const kernelProver = new KernelProver(kernelOracle);
    this.log(`Executing kernel prover...`);
    const { proof, publicInputs } = await kernelProver.prove(txExecutionRequest.toTxRequest(), executionResult);
    this.log('Proof completed!');

    const newContractPublicFunctions = newContract ? getNewContractPublicFunctions(newContract) : [];

    const encryptedLogs = new TxL2Logs(collectEncryptedLogs(executionResult));
    const unencryptedLogs = new TxL2Logs(collectUnencryptedLogs(executionResult));
    const enqueuedPublicFunctions = collectEnqueuedPublicFunctionCalls(executionResult);

    return new Tx(
      publicInputs,
      proof,
      encryptedLogs,
      unencryptedLogs,
      newContractPublicFunctions,
      enqueuedPublicFunctions,
    );
  }

  /**
   * Returns true if the account specified by the given address is synched to the latest block
   * @param account - The aztec address for which to query the sync status
   * @returns True if the account is fully synched, false otherwise
   */
  public async isAccountSynchronised(account: AztecAddress) {
    return await this.synchroniser.isAccountSynchronised(account);
  }
}
