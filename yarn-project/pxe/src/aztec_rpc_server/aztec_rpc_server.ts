import {
  AcirSimulator,
  ExecutionResult,
  collectEncryptedLogs,
  collectEnqueuedPublicFunctionCalls,
  collectUnencryptedLogs,
  resolveOpcodeLocations,
} from '@aztec/acir-simulator';
import {
  AztecAddress,
  CircuitsWasm,
  CompleteAddress,
  EthAddress,
  FunctionData,
  GrumpkinPrivateKey,
  KernelCircuitPublicInputsFinal,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  PartialAddress,
  PublicCallRequest,
} from '@aztec/circuits.js';
import { computeCommitmentNonce, siloNullifier } from '@aztec/circuits.js/abis';
import { encodeArguments } from '@aztec/foundation/abi';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr, Point } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import NoirVersion from '@aztec/noir-compiler/noir-version';
import {
  AuthWitness,
  AztecNode,
  AztecRPC,
  ContractDao,
  ContractData,
  DeployedContract,
  ExtendedContractData,
  FunctionCall,
  INITIAL_L2_BLOCK_NUM,
  KeyStore,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  LogType,
  NodeInfo,
  NotePreimage,
  PublicKey,
  SimulationError,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxL2Logs,
  TxReceipt,
  TxStatus,
  getNewContractPublicFunctions,
  isNoirCallStackUnresolved,
  toContractDao,
} from '@aztec/types';

import { RpcServerConfig, getPackageInfo } from '../config/index.js';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { Database } from '../database/index.js';
import { KernelOracle } from '../kernel_oracle/index.js';
import { KernelProver } from '../kernel_prover/kernel_prover.js';
import { getAcirSimulator } from '../simulator/index.js';
import { Synchronizer } from '../synchronizer/index.js';

/**
 * A remote Aztec RPC Client implementation.
 */
export class AztecRPCServer implements AztecRPC {
  private synchronizer: Synchronizer;
  private contractDataOracle: ContractDataOracle;
  private simulator: AcirSimulator;
  private log: DebugLogger;
  private sandboxVersion: string;

  constructor(
    private keyStore: KeyStore,
    private node: AztecNode,
    private db: Database,
    private config: RpcServerConfig,
    logSuffix?: string,
  ) {
    this.log = createDebugLogger(logSuffix ? `aztec:rpc_server_${logSuffix}` : `aztec:rpc_server`);
    this.synchronizer = new Synchronizer(node, db, logSuffix);
    this.contractDataOracle = new ContractDataOracle(db, node);
    this.simulator = getAcirSimulator(db, node, node, node, keyStore, this.contractDataOracle);

    this.sandboxVersion = getPackageInfo().version;
  }

  /**
   * Starts the Aztec RPC server by beginning the synchronisation process between the Aztec node and the database.
   *
   * @returns A promise that resolves when the server has started successfully.
   */
  public async start() {
    await this.synchronizer.start(INITIAL_L2_BLOCK_NUM, 1, this.config.l2BlockPollingIntervalMS);
    const info = await this.getNodeInfo();
    this.log.info(`Started RPC server connected to chain ${info.chainId} version ${info.protocolVersion}`);
  }

  /**
   * Stops the Aztec RPC server, halting processing of new transactions and shutting down the synchronizer.
   * This function ensures that all ongoing tasks are completed before stopping the server.
   * It is useful for gracefully shutting down the server during maintenance or restarts.
   *
   * @returns A Promise resolving once the server has been stopped successfully.
   */
  public async stop() {
    await this.synchronizer.stop();
    this.log.info('Stopped');
  }

  public addAuthWitness(witness: AuthWitness) {
    return this.db.addAuthWitness(witness.requestHash, witness.witness);
  }

  public async registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress) {
    const completeAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(privKey, partialAddress);
    const wasAdded = await this.db.addCompleteAddress(completeAddress);
    if (wasAdded) {
      const pubKey = this.keyStore.addAccount(privKey);
      this.synchronizer.addAccount(pubKey, this.keyStore);
      this.log.info(`Registered account ${completeAddress.address.toString()}`);
      this.log.debug(`Registered account\n ${completeAddress.toReadableString()}`);
    } else {
      this.log.info(`Account:\n "${completeAddress.address.toString()}"\n already registered.`);
    }
  }

  public async getRegisteredAccounts(): Promise<CompleteAddress[]> {
    // Get complete addresses of both the recipients and the accounts
    const addresses = await this.db.getCompleteAddresses();
    // Filter out the addresses not corresponding to accounts
    const accountPubKeys = await this.keyStore.getAccounts();
    const accounts = addresses.filter(address => accountPubKeys.find(pubKey => pubKey.equals(address.publicKey)));
    return accounts;
  }

  public async getRegisteredAccount(address: AztecAddress): Promise<CompleteAddress | undefined> {
    const result = await this.getRegisteredAccounts();
    const account = result.find(r => r.address.equals(address));
    return Promise.resolve(account);
  }

  public async registerRecipient(recipient: CompleteAddress): Promise<void> {
    const wasAdded = await this.db.addCompleteAddress(recipient);
    if (wasAdded) {
      this.log.info(`Added recipient:\n ${recipient.toReadableString()}`);
    } else {
      this.log.info(`Recipient:\n "${recipient.toReadableString()}"\n already registered.`);
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
    const contractDaos = contracts.map(c => toContractDao(c.abi, c.completeAddress, c.portalContract));
    await Promise.all(contractDaos.map(c => this.db.addContract(c)));
    for (const contract of contractDaos) {
      const portalInfo =
        contract.portalContract && !contract.portalContract.isZero() ? ` with portal ${contract.portalContract}` : '';
      this.log.info(`Added contract ${contract.name} at ${contract.completeAddress.address}${portalInfo}`);
    }
  }

  public async getContracts(): Promise<AztecAddress[]> {
    return (await this.db.getContracts()).map(c => c.completeAddress.address);
  }

  public async getPublicStorageAt(contract: AztecAddress, storageSlot: Fr) {
    if ((await this.getContractData(contract)) === undefined) {
      throw new Error(`Contract ${contract.toString()} is not deployed`);
    }
    return await this.node.getPublicStorageAt(contract, storageSlot.value);
  }

  public async getPrivateStorageAt(owner: AztecAddress, contract: AztecAddress, storageSlot: Fr) {
    if ((await this.getContractData(contract)) === undefined) {
      throw new Error(`Contract ${contract.toString()} is not deployed`);
    }
    const notes = await this.db.getNoteSpendingInfo(contract, storageSlot);
    const ownerCompleteAddress = await this.db.getCompleteAddress(owner);
    if (!ownerCompleteAddress) throw new Error(`Owner ${owner} not registered in RPC server`);
    const { publicKey: ownerPublicKey } = ownerCompleteAddress;
    const ownerNotes = notes.filter(n => n.publicKey.equals(ownerPublicKey));
    return ownerNotes.map(n => n.notePreimage);
  }

  public async addNote(
    contractAddress: AztecAddress,
    storageSlot: Fr,
    preimage: NotePreimage,
    nonce: Fr,
    account: PublicKey,
  ) {
    const { innerNoteHash, siloedNoteHash, uniqueSiloedNoteHash, innerNullifier } =
      await this.simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage.items);

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
    // This can always be `uniqueSiloedNoteHash` once notes added from public also include nonces.
    const noteHashToLookUp = nonce.isZero() ? siloedNoteHash : uniqueSiloedNoteHash;
    const index = await this.node.findCommitmentIndex(noteHashToLookUp.toBuffer());
    if (index === undefined) {
      throw new Error('Note does not exist.');
    }

    const wasm = await CircuitsWasm.get();
    const siloedNullifier = siloNullifier(wasm, contractAddress, innerNullifier!);
    const nullifierIndex = await this.node.findNullifierIndex(siloedNullifier);
    if (nullifierIndex !== undefined) {
      throw new Error('The note has been destroyed.');
    }

    // TODO - Should not modify the db while syncing.
    await this.db.addNoteSpendingInfo({
      contractAddress,
      storageSlot,
      notePreimage: preimage,
      nonce,
      innerNoteHash,
      siloedNullifier,
      index,
      publicKey: account,
    });
  }

  public async getNoteNonces(
    contractAddress: AztecAddress,
    storageSlot: Fr,
    preimage: NotePreimage,
    txHash: TxHash,
  ): Promise<Fr[]> {
    const tx = await this.node.getTx(txHash);
    if (!tx) {
      throw new Error(`Unknown tx: ${txHash}`);
    }

    const wasm = await CircuitsWasm.get();

    const nonces: Fr[] = [];
    const firstNullifier = tx.newNullifiers[0];
    const commitments = tx.newCommitments;
    for (let i = 0; i < commitments.length; ++i) {
      const commitment = commitments[i];
      if (commitment.equals(Fr.ZERO)) break;

      const nonce = computeCommitmentNonce(wasm, firstNullifier, i);
      const { uniqueSiloedNoteHash } = await this.simulator.computeNoteHashAndNullifier(
        contractAddress,
        nonce,
        storageSlot,
        preimage.items,
      );
      if (commitment.equals(uniqueSiloedNoteHash)) {
        nonces.push(nonce);
      }
    }
    return nonces;
  }

  public async getBlock(blockNumber: number): Promise<L2Block | undefined> {
    // If a negative block number is provided the current block number is fetched.
    if (blockNumber < 0) {
      blockNumber = await this.node.getBlockNumber();
    }
    return await this.node.getBlock(blockNumber);
  }

  public async simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean) {
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
    if (simulatePublic) await this.#simulatePublicCalls(tx);
    this.log.info(`Executed local simulation for ${await tx.getTxHash()}`);

    return tx;
  }

  public async sendTx(tx: Tx): Promise<TxHash> {
    const txHash = await tx.getTxHash();
    if (await this.node.getTx(txHash)) {
      throw new Error(`A settled tx with equal hash ${txHash.toString()} exists.`);
    }
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
    const settledTx = await this.node.getTx(txHash);
    if (settledTx) {
      const deployedContractAddress = settledTx.newContractData.find(
        c => !c.contractAddress.equals(AztecAddress.ZERO),
      )?.contractAddress;

      return new TxReceipt(
        txHash,
        TxStatus.MINED,
        '',
        settledTx.blockHash,
        settledTx.blockNumber,
        deployedContractAddress,
      );
    }

    const pendingTx = await this.node.getPendingTxByHash(txHash);
    if (pendingTx) {
      return new TxReceipt(txHash, TxStatus.PENDING, '');
    }

    return new TxReceipt(txHash, TxStatus.DROPPED, 'Tx dropped by P2P node.');
  }

  public async getTx(txHash: TxHash): Promise<L2Tx | undefined> {
    return await this.node.getTx(txHash);
  }

  async getBlockNumber(): Promise<number> {
    return await this.node.getBlockNumber();
  }

  public async getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    return await this.node.getExtendedContractData(contractAddress);
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
    const [version, chainId, contractAddresses] = await Promise.all([
      this.node.getVersion(),
      this.node.getChainId(),
      this.node.getL1ContractAddresses(),
    ]);

    const nodeInfo: NodeInfo = {
      sandboxVersion: this.sandboxVersion,
      compatibleNargoVersion: NoirVersion.tag,
      chainId,
      protocolVersion: version,
      l1ContractAddresses: contractAddresses,
    };
    return nodeInfo;
  }

  /**
   * Retrieves the simulation parameters required to run an ACIR simulation.
   * This includes the contract address, function ABI, portal contract address, and historic tree roots.
   *
   * @param execRequest - The transaction request object containing details of the contract call.
   * @returns An object containing the contract address, function ABI, portal contract address, and historic tree roots.
   */
  async #getSimulationParameters(execRequest: FunctionCall | TxExecutionRequest) {
    const contractAddress = (execRequest as FunctionCall).to ?? (execRequest as TxExecutionRequest).origin;
    const functionAbi = await this.contractDataOracle.getFunctionAbi(
      contractAddress,
      execRequest.functionData.selector,
    );
    const debug = await this.contractDataOracle.getFunctionDebugMetadata(
      contractAddress,
      execRequest.functionData.selector,
    );
    const portalContract = await this.contractDataOracle.getPortalContractAddress(contractAddress);

    return {
      contractAddress,
      functionAbi: {
        ...functionAbi,
        debug,
      },
      portalContract,
    };
  }

  async #simulate(txRequest: TxExecutionRequest): Promise<ExecutionResult> {
    // TODO - Pause syncing while simulating.

    const { contractAddress, functionAbi, portalContract } = await this.#getSimulationParameters(txRequest);

    this.log('Executing simulator...');
    try {
      const result = await this.simulator.run(txRequest, functionAbi, contractAddress, portalContract);
      this.log('Simulation completed!');
      return result;
    } catch (err) {
      if (err instanceof SimulationError) {
        await this.#enrichSimulationError(err);
      }
      throw err;
    }
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
    const { contractAddress, functionAbi, portalContract } = await this.#getSimulationParameters(execRequest);

    this.log('Executing unconstrained simulator...');
    try {
      const result = await this.simulator.runUnconstrained(
        execRequest,
        from ?? AztecAddress.ZERO,
        functionAbi,
        contractAddress,
        portalContract,
        this.node,
      );
      this.log('Unconstrained simulation completed!');

      return result;
    } catch (err) {
      if (err instanceof SimulationError) {
        await this.#enrichSimulationError(err);
      }
      throw err;
    }
  }

  /**
   * Simulate the public part of a transaction.
   * This allows to catch public execution errors before submitting the transaction.
   * It can also be used for estimating gas in the future.
   * @param tx - The transaction to be simulated.
   */
  async #simulatePublicCalls(tx: Tx) {
    try {
      await this.node.simulatePublicCalls(tx);
    } catch (err) {
      // Try to fill in the noir call stack since the RPC server may have access to the debug metadata
      if (err instanceof SimulationError) {
        const callStack = err.getCallStack();
        const originalFailingFunction = callStack[callStack.length - 1];
        const debugInfo = await this.contractDataOracle.getFunctionDebugMetadata(
          originalFailingFunction.contractAddress,
          originalFailingFunction.functionSelector,
        );
        const noirCallStack = err.getNoirCallStack();
        if (debugInfo && isNoirCallStackUnresolved(noirCallStack)) {
          const parsedCallStack = resolveOpcodeLocations(noirCallStack, debugInfo);
          err.setNoirCallStack(parsedCallStack);
        }
        await this.#enrichSimulationError(err);
      }

      throw err;
    }
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

    // Get values that allow us to reconstruct the block hash
    const executionResult = await this.#simulate(txExecutionRequest);

    const kernelOracle = new KernelOracle(this.contractDataOracle, this.node);
    const kernelProver = new KernelProver(kernelOracle);
    this.log(`Executing kernel prover...`);
    const { proof, publicInputs } = await kernelProver.prove(txExecutionRequest.toTxRequest(), executionResult);

    const newContractPublicFunctions = newContract ? getNewContractPublicFunctions(newContract) : [];

    const encryptedLogs = new TxL2Logs(collectEncryptedLogs(executionResult));
    const unencryptedLogs = new TxL2Logs(collectUnencryptedLogs(executionResult));
    const enqueuedPublicFunctions = collectEnqueuedPublicFunctionCalls(executionResult);

    const contractData = new ContractData(newContract?.completeAddress.address ?? AztecAddress.ZERO, EthAddress.ZERO);
    const extendedContractData = new ExtendedContractData(
      contractData,
      newContractPublicFunctions,
      newContract?.completeAddress.partialAddress ?? Fr.ZERO,
      newContract?.completeAddress.publicKey ?? Point.ZERO,
    );

    // HACK(#1639): Manually patches the ordering of the public call stack
    // TODO(#757): Enforce proper ordering of enqueued public calls
    await this.patchPublicCallStackOrdering(publicInputs, enqueuedPublicFunctions);

    return new Tx(publicInputs, proof, encryptedLogs, unencryptedLogs, enqueuedPublicFunctions, [extendedContractData]);
  }

  /**
   * Adds contract and function names to a simulation error.
   * @param err - The error to enrich.
   */
  async #enrichSimulationError(err: SimulationError) {
    // Maps contract addresses to the set of functions selectors that were in error.
    // Using strings because map and set don't use .equals()
    const mentionedFunctions: Map<string, Set<string>> = new Map();

    err.getCallStack().forEach(({ contractAddress, functionSelector }) => {
      if (!mentionedFunctions.has(contractAddress.toString())) {
        mentionedFunctions.set(contractAddress.toString(), new Set());
      }
      mentionedFunctions.get(contractAddress.toString())!.add(functionSelector.toString());
    });

    await Promise.all(
      [...mentionedFunctions.entries()].map(async ([contractAddress, selectors]) => {
        const parsedContractAddress = AztecAddress.fromString(contractAddress);
        const contract = await this.db.getContract(parsedContractAddress);
        if (contract) {
          err.enrichWithContractName(parsedContractAddress, contract.name);
          selectors.forEach(selector => {
            const functionAbi = contract.functions.find(f => f.selector.toString() === selector);
            if (functionAbi) {
              err.enrichWithFunctionName(parsedContractAddress, functionAbi.selector, functionAbi.name);
            }
          });
        }
      }),
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
    publicInputs: KernelCircuitPublicInputsFinal,
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

  public async isGlobalStateSynchronized() {
    return await this.synchronizer.isGlobalStateSynchronized();
  }

  public async isAccountStateSynchronized(account: AztecAddress) {
    return await this.synchronizer.isAccountStateSynchronized(account);
  }

  public getSyncStatus() {
    return Promise.resolve(this.synchronizer.getSyncStatus());
  }
}
