import {
  ExecutionResult,
  collectEncryptedLogs,
  collectEnqueuedPublicFunctionCalls,
  collectUnencryptedLogs,
} from '@aztec/acir-simulator';
import {
  AztecAddress,
  CompleteAddress,
  FunctionData,
  KernelCircuitPublicInputs,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  PartialAddress,
  PrivateKey,
  PublicCallRequest,
} from '@aztec/circuits.js';
import { encodeArguments } from '@aztec/foundation/abi';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import {
  AztecNode,
  AztecRPC,
  ContractDao,
  ContractData,
  ContractDataAndBytecode,
  DeployedContract,
  FunctionCall,
  INITIAL_L2_BLOCK_NUM,
  KeyStore,
  L2Block,
  L2BlockL2Logs,
  LogType,
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
    logSuffix?: string,
  ) {
    this.log = createDebugLogger(logSuffix ? `aztec:rpc_server_${logSuffix}` : `aztec:rpc_server`);
    this.synchroniser = new Synchroniser(node, db, logSuffix);
  }

  /**
   * Starts the Aztec RPC server by beginning the synchronisation process between the Aztec node and the database.
   *
   * @returns A promise that resolves when the server has started successfully.
   */
  public async start() {
    await this.synchroniser.start(INITIAL_L2_BLOCK_NUM, 1, this.config.l2BlockPollingIntervalMS);
    const info = await this.getNodeInfo();
    this.log.info(`Started RPC server connected to chain ${info.chainId} version ${info.version}`);
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
    this.log.info('Stopped');
  }

  public async registerAccount(privKey: PrivateKey, partialAddress: PartialAddress) {
    const completeAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(privKey, partialAddress);
    const wasAdded = await this.db.addCompleteAddress(completeAddress);
    if (wasAdded) {
      const pubKey = this.keyStore.addAccount(privKey);
      this.synchroniser.addAccount(pubKey, this.keyStore);
      this.log.info(`Added account: ${completeAddress.toReadableString()}`);
    } else {
      this.log.info(`Account "${completeAddress.toReadableString()}" already registered.`);
    }
  }

  public async getAccounts(): Promise<CompleteAddress[]> {
    // Get complete addresses of both the recipients and the accounts
    const addresses = await this.db.getCompleteAddresses();
    // Filter out the addresses not corresponding to accounts
    const accountPubKeys = await this.keyStore.getAccounts();
    const accounts = addresses.filter(address => accountPubKeys.find(pubKey => pubKey.equals(address.publicKey)));
    return accounts;
  }

  public async getAccount(address: AztecAddress): Promise<CompleteAddress | undefined> {
    const result = await this.getAccounts();
    const account = result.find(r => r.address.equals(address));
    return Promise.resolve(account);
  }

  public async registerRecipient(recipient: CompleteAddress): Promise<void> {
    const wasAdded = await this.db.addCompleteAddress(recipient);
    if (wasAdded) {
      this.log.info(`Added recipient: ${recipient.toReadableString()}`);
    } else {
      this.log.info(`Recipient "${recipient.toReadableString()}" already registered.`);
    }
  }

  public async getRecipients(): Promise<CompleteAddress[]> {
    // Get complete addresses of both the recipients and the accounts
    const addresses = await this.db.getCompleteAddresses();
    // Filter out the addresses corresponding to accounts
    const accountPubKeys = await this.keyStore.getAccounts();
    const recipients = addresses.filter(address => !accountPubKeys.find(pubKey => pubKey.equals(address.publicKey)));
    return recipients;
  }

  public async getRecipient(address: AztecAddress): Promise<CompleteAddress | undefined> {
    const result = await this.getRecipients();
    const recipient = result.find(r => r.address.equals(address));
    return Promise.resolve(recipient);
  }

  public async addContracts(contracts: DeployedContract[]) {
    const contractDaos = contracts.map(c => toContractDao(c.abi, c.address, c.portalContract));
    await Promise.all(contractDaos.map(c => this.db.addContract(c)));
    for (const contract of contractDaos) {
      const portalInfo =
        contract.portalContract && !contract.portalContract.isZero() ? ` with portal ${contract.portalContract}` : '';
      this.log.info(`Added contract ${contract.name} at ${contract.address}${portalInfo}`);
    }
  }

  public async getContracts(): Promise<AztecAddress[]> {
    return (await this.db.getContracts()).map(c => c.address);
  }

  public async getPublicStorageAt(contract: AztecAddress, storageSlot: Fr) {
    if ((await this.getContractData(contract)) === undefined) {
      throw new Error(`Contract ${contract.toString()} is not deployed`);
    }
    return await this.node.getPublicStorageAt(contract, storageSlot.value);
  }

  public async getBlock(blockNumber: number): Promise<L2Block | undefined> {
    // If a negative block number is provided the current block number is fetched.
    if (blockNumber < 0) {
      blockNumber = await this.node.getBlockNumber();
    }
    return await this.node.getBlock(blockNumber);
  }

  public async simulateTx(txRequest: TxExecutionRequest) {
    if (!txRequest.functionData.isPrivate) {
      throw new Error(`Public entrypoints are not allowed`);
    }
    if (txRequest.functionData.isInternal === undefined) {
      throw new Error(`Unspecified internal are not allowed`);
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

    this.log.info(`Executed local simulation for ${await tx.getTxHash()}`);
    return tx;
  }

  public async sendTx(tx: Tx): Promise<TxHash> {
    const txHash = await tx.getTxHash();
    this.log.info(`Sending transaction ${txHash}`);
    await this.node.sendTx(tx);
    return txHash;
  }

  public async viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress) {
    const functionCall = await this.#getFunctionCall(functionName, args, to);
    const executionResult = await this.#simulateUnconstrained(functionCall, from);

    // TODO - Return typed result based on the function abi.
    return executionResult;
  }

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

    const isSynchronised = await this.synchroniser.isGlobalStateSynchronised();
    if (!isSynchronised) {
      // there is a pending L2 block, which means the transaction will not be in the tx pool but may be awaiting mine on L1
      return partialReceipt;
    }

    // TODO we should refactor this once the node can store transactions. At that point we should query the node and not deal with block heights.
    partialReceipt.status = TxStatus.DROPPED;
    partialReceipt.error = 'Tx dropped by P2P node.';
    return partialReceipt;
  }

  async getBlockNumber(): Promise<number> {
    return await this.node.getBlockNumber();
  }

  public async getContractDataAndBytecode(contractAddress: AztecAddress): Promise<ContractDataAndBytecode | undefined> {
    return await this.node.getContractDataAndBytecode(contractAddress);
  }

  public async getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return await this.node.getContractData(contractAddress);
  }

  public async getUnencryptedLogs(from: number, limit: number): Promise<L2BlockL2Logs[]> {
    return await this.node.getLogs(from, limit, LogType.UNENCRYPTED);
  }

  async #getFunctionCall(functionName: string, args: any[], to: AztecAddress): Promise<FunctionCall> {
    const contract = await this.db.getContract(to);
    if (!contract) {
      throw new Error(`Unknown contract ${to}: add it to Aztec RPC server by calling server.addContracts(...)`);
    }

    const functionDao = contract.functions.find(f => f.name === functionName);
    if (!functionDao) {
      throw new Error(`Unknown function ${functionName} in contract ${contract.name}.`);
    }

    return {
      args: encodeArguments(functionDao, args),
      functionData: FunctionData.fromAbi(functionDao),
      to,
    };
  }

  public async getNodeInfo(): Promise<NodeInfo> {
    const [version, chainId, rollupAddress] = await Promise.all([
      this.node.getVersion(),
      this.node.getChainId(),
      this.node.getRollupAddress(),
    ]);

    return {
      version,
      chainId,
      rollupAddress,
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
    execRequest: FunctionCall | TxExecutionRequest,
    contractDataOracle: ContractDataOracle,
  ) {
    const contractAddress = (execRequest as FunctionCall).to ?? (execRequest as TxExecutionRequest).origin;
    const functionAbi = await contractDataOracle.getFunctionAbi(
      contractAddress,
      execRequest.functionData.functionSelectorBuffer,
    );
    const debug = await contractDataOracle.getFunctionDebugMetadata(
      contractAddress,
      execRequest.functionData.functionSelectorBuffer,
    );
    const portalContract = await contractDataOracle.getPortalContractAddress(contractAddress);

    return {
      contractAddress,
      functionAbi: {
        ...functionAbi,
        debug,
      },
      portalContract,
    };
  }

  async #simulate(txRequest: TxExecutionRequest, contractDataOracle?: ContractDataOracle): Promise<ExecutionResult> {
    // TODO - Pause syncing while simulating.
    if (!contractDataOracle) {
      contractDataOracle = new ContractDataOracle(this.db, this.node);
    }

    const { contractAddress, functionAbi, portalContract } = await this.#getSimulationParameters(
      txRequest,
      contractDataOracle,
    );

    const simulator = getAcirSimulator(this.db, this.node, this.node, this.node, this.keyStore, contractDataOracle);

    this.log('Executing simulator...');
    const result = await simulator.run(txRequest, functionAbi, contractAddress, portalContract);
    this.log('Simulation completed!');

    return result;
  }

  /**
   * Simulate an unconstrained transaction on the given contract, without considering constraints set by ACIR.
   * The simulation parameters are fetched using ContractDataOracle and executed using AcirSimulator.
   * Returns the simulation result containing the outputs of the unconstrained function.
   *
   * @param execRequest - The transaction request object containing the target contract and function data.
   * @param from - The origin of the request.
   * @returns The simulation result containing the outputs of the unconstrained function.
   */
  async #simulateUnconstrained(execRequest: FunctionCall, from?: AztecAddress) {
    const contractDataOracle = new ContractDataOracle(this.db, this.node);

    const { contractAddress, functionAbi, portalContract } = await this.#getSimulationParameters(
      execRequest,
      contractDataOracle,
    );

    const simulator = getAcirSimulator(this.db, this.node, this.node, this.node, this.keyStore, contractDataOracle);

    this.log('Executing unconstrained simulator...');
    const result = await simulator.runUnconstrained(
      execRequest,
      from ?? AztecAddress.ZERO,
      functionAbi,
      contractAddress,
      portalContract,
      this.node,
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

    // Get values that allow us to reconstruct the block hash
    const executionResult = await this.#simulate(txExecutionRequest, contractDataOracle);

    const kernelProver = new KernelProver(kernelOracle);
    this.log(`Executing kernel prover...`);
    const { proof, publicInputs } = await kernelProver.prove(txExecutionRequest.toTxRequest(), executionResult);

    const newContractPublicFunctions = newContract ? getNewContractPublicFunctions(newContract) : [];

    const encryptedLogs = new TxL2Logs(collectEncryptedLogs(executionResult));
    const unencryptedLogs = new TxL2Logs(collectUnencryptedLogs(executionResult));
    const enqueuedPublicFunctions = collectEnqueuedPublicFunctionCalls(executionResult);

    // HACK(#1639): Manually patches the ordering of the public call stack
    // TODO(#757): Enforce proper ordering of enqueued public calls
    await this.patchPublicCallStackOrdering(publicInputs, enqueuedPublicFunctions);

    return new Tx(
      publicInputs,
      proof,
      encryptedLogs,
      unencryptedLogs,
      newContractPublicFunctions,
      enqueuedPublicFunctions,
    );
  }

  // HACK(#1639): this is a hack to fix ordering of public calls enqueued in the call stack. Since the private kernel
  // cannot keep track of side effects that happen after or before a nested call, we override the public call stack
  // it emits with whatever we got from the simulator collected enqueued calls. As a sanity check, we at least verify
  // that the elements are the same, so we are only tweaking their ordering.
  // See yarn-project/end-to-end/src/e2e_ordering.test.ts
  // See https://github.com/AztecProtocol/aztec-packages/issues/1615
  // TODO(#757): Enforce proper ordering of enqueued public calls
  private async patchPublicCallStackOrdering(
    publicInputs: KernelCircuitPublicInputs,
    enqueuedPublicCalls: PublicCallRequest[],
  ) {
    const callToHash = (call: PublicCallRequest) => call.toPublicCallStackItem().then(item => item.hash());
    const enqueuedPublicCallsHashes = await Promise.all(enqueuedPublicCalls.map(callToHash));
    const { publicCallStack } = publicInputs.end;

    // Validate all items in enqueued public calls are in the kernel emitted stack
    const areEqual = enqueuedPublicCallsHashes.reduce(
      (accum, enqueued) => accum && !!publicCallStack.find(item => item.equals(enqueued)),
      true,
    );

    if (!areEqual) {
      throw new Error(
        `Enqueued public function calls and public call stack do not match.\nEnqueued calls: ${enqueuedPublicCallsHashes
          .map(h => h.toString())
          .join(', ')}\nPublic call stack: ${publicCallStack.map(i => i.toString()).join(', ')}`,
      );
    }

    // Override kernel output
    publicInputs.end.publicCallStack = padArrayEnd(
      enqueuedPublicCallsHashes,
      Fr.ZERO,
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    );
  }

  public async isGlobalStateSynchronised() {
    return await this.synchroniser.isGlobalStateSynchronised();
  }

  public async isAccountStateSynchronised(account: AztecAddress) {
    return await this.synchroniser.isAccountStateSynchronised(account);
  }

  public getSyncStatus() {
    return Promise.resolve(this.synchroniser.getSyncStatus());
  }
}
