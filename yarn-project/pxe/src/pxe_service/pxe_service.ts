import {
  type AuthWitness,
  type AztecNode,
  EncryptedTxL2Logs,
  ExtendedNote,
  type FunctionCall,
  type GetUnencryptedLogsResponse,
  type KeyStore,
  type L2Block,
  type LogFilter,
  MerkleTreeId,
  type NoteFilter,
  type PXE,
  SimulatedTx,
  SimulationError,
  Tx,
  type TxEffect,
  type TxExecutionRequest,
  type TxHash,
  type TxReceipt,
  UnencryptedTxL2Logs,
  isNoirCallStackUnresolved,
} from '@aztec/circuit-types';
import { type TxPXEProcessingStats } from '@aztec/circuit-types/stats';
import {
  AztecAddress,
  CallRequest,
  CompleteAddress,
  FunctionData,
  type GrumpkinPrivateKey,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  type PartialAddress,
  type PrivateKernelTailCircuitPublicInputs,
  type PublicCallRequest,
  computeContractClassId,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import { computeCommitmentNonce, siloNullifier } from '@aztec/circuits.js/hash';
import { type ContractArtifact, type DecodedReturn, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { arrayNonEmptyLength, padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { SerialQueue } from '@aztec/foundation/fifo';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import {
  type AcirSimulator,
  type ExecutionResult,
  collectEncryptedLogs,
  collectEnqueuedPublicFunctionCalls,
  collectUnencryptedLogs,
  resolveOpcodeLocations,
} from '@aztec/simulator';
import { type ContractClassWithId, type ContractInstanceWithAddress } from '@aztec/types/contracts';
import { type NodeInfo } from '@aztec/types/interfaces';

import { type PXEServiceConfig, getPackageInfo } from '../config/index.js';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { type PxeDatabase } from '../database/index.js';
import { NoteDao } from '../database/note_dao.js';
import { KernelOracle } from '../kernel_oracle/index.js';
import { KernelProver } from '../kernel_prover/kernel_prover.js';
import { getAcirSimulator } from '../simulator/index.js';
import { Synchronizer } from '../synchronizer/index.js';

/**
 * A Private eXecution Environment (PXE) implementation.
 */
export class PXEService implements PXE {
  private synchronizer: Synchronizer;
  private contractDataOracle: ContractDataOracle;
  private simulator: AcirSimulator;
  private log: DebugLogger;
  private nodeVersion: string;
  // serialize synchronizer and calls to proveTx.
  // ensures that state is not changed while simulating
  private jobQueue = new SerialQueue();

  constructor(
    private keyStore: KeyStore,
    private node: AztecNode,
    private db: PxeDatabase,
    private config: PXEServiceConfig,
    logSuffix?: string,
  ) {
    this.log = createDebugLogger(logSuffix ? `aztec:pxe_service_${logSuffix}` : `aztec:pxe_service`);
    this.synchronizer = new Synchronizer(node, db, this.jobQueue, logSuffix);
    this.contractDataOracle = new ContractDataOracle(db);
    this.simulator = getAcirSimulator(db, node, keyStore, this.contractDataOracle);
    this.nodeVersion = getPackageInfo().version;

    this.jobQueue.start();
  }

  /**
   * Starts the PXE Service by beginning the synchronization process between the Aztec node and the database.
   *
   * @returns A promise that resolves when the server has started successfully.
   */
  public async start() {
    const { l2BlockPollingIntervalMS } = this.config;
    await this.synchronizer.start(1, l2BlockPollingIntervalMS);
    await this.restoreNoteProcessors();
    const info = await this.getNodeInfo();
    this.log.info(`Started PXE connected to chain ${info.chainId} version ${info.protocolVersion}`);
  }

  private async restoreNoteProcessors() {
    const publicKeys = await this.keyStore.getAccounts();
    const publicKeysSet = new Set(publicKeys.map(k => k.toString()));

    const registeredAddresses = await this.db.getCompleteAddresses();

    let count = 0;
    for (const address of registeredAddresses) {
      if (!publicKeysSet.has(address.publicKey.toString())) {
        continue;
      }

      count++;
      this.synchronizer.addAccount(address.publicKey, this.keyStore, this.config.l2StartingBlock);
    }

    if (count > 0) {
      this.log(`Restored ${count} accounts`);
    }
  }

  /**
   * Stops the PXE Service, halting processing of new transactions and shutting down the synchronizer.
   * This function ensures that all ongoing tasks are completed before stopping the server.
   * It is useful for gracefully shutting down the server during maintenance or restarts.
   *
   * @returns A Promise resolving once the server has been stopped successfully.
   */
  public async stop() {
    await this.jobQueue.cancel();
    this.log.info('Cancelled Job Queue');
    await this.synchronizer.stop();
    this.log.info('Stopped Synchronizer');
  }

  /** Returns an estimate of the db size in bytes. */
  public estimateDbSize() {
    return this.db.estimateSize();
  }

  public addAuthWitness(witness: AuthWitness) {
    return this.db.addAuthWitness(witness.requestHash, witness.witness);
  }

  public getAuthWitness(messageHash: Fr): Promise<Fr[] | undefined> {
    return this.db.getAuthWitness(messageHash);
  }

  public addCapsule(capsule: Fr[]) {
    return this.db.addCapsule(capsule);
  }

  public getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return this.db.getContractInstance(address);
  }

  public async getContractClass(id: Fr): Promise<ContractClassWithId | undefined> {
    const artifact = await this.db.getContractArtifact(id);
    return artifact && getContractClassFromArtifact(artifact);
  }

  public async registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress): Promise<CompleteAddress> {
    const completeAddress = CompleteAddress.fromPrivateKeyAndPartialAddress(privKey, partialAddress);
    const wasAdded = await this.db.addCompleteAddress(completeAddress);
    if (wasAdded) {
      const pubKey = await this.keyStore.addAccount(privKey);
      this.synchronizer.addAccount(pubKey, this.keyStore, this.config.l2StartingBlock);
      this.log.info(`Registered account ${completeAddress.address.toString()}`);
      this.log.debug(`Registered account\n ${completeAddress.toReadableString()}`);
    } else {
      this.log.info(`Account:\n "${completeAddress.address.toString()}"\n already registered.`);
    }
    return completeAddress;
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

  public async registerContractClass(artifact: ContractArtifact): Promise<void> {
    const contractClassId = computeContractClassId(getContractClassFromArtifact(artifact));
    await this.db.addContractArtifact(contractClassId, artifact);
    this.log.info(`Added contract class ${artifact.name} with id ${contractClassId}`);
  }

  public async registerContract(contract: { instance: ContractInstanceWithAddress; artifact?: ContractArtifact }) {
    const { instance } = contract;
    let { artifact } = contract;

    if (artifact) {
      // If the user provides an artifact, validate it against the expected class id and register it
      const contractClassId = computeContractClassId(getContractClassFromArtifact(artifact));
      if (!contractClassId.equals(instance.contractClassId)) {
        throw new Error(
          `Artifact does not match expected class id (computed ${contractClassId} but instance refers to ${instance.contractClassId})`,
        );
      }
      await this.db.addContractArtifact(contractClassId, artifact);
    } else {
      // Otherwise, make sure there is an artifact already registered for that class id
      artifact = await this.db.getContractArtifact(instance.contractClassId);
      if (!artifact) {
        throw new Error(
          `Missing contract artifact for class id ${instance.contractClassId} for contract ${instance.address}`,
        );
      }
    }

    this.log.info(`Added contract ${artifact.name} at ${instance.address.toString()}`);
    await this.db.addContractInstance(instance);
    await this.synchronizer.reprocessDeferredNotesForContract(instance.address);
  }

  public getContracts(): Promise<AztecAddress[]> {
    return this.db.getContractsAddresses();
  }

  public async getPublicStorageAt(contract: AztecAddress, slot: Fr) {
    if (!(await this.getContractInstance(contract))) {
      throw new Error(`Contract ${contract.toString()} is not deployed`);
    }
    return await this.node.getPublicStorageAt(contract, slot);
  }

  public async getNotes(filter: NoteFilter): Promise<ExtendedNote[]> {
    const noteDaos = await this.db.getNotes(filter);

    // TODO(benesjan): Refactor --> This type conversion is ugly but I decided to keep it this way for now because
    // key derivation will affect all this
    const extendedNotes = noteDaos.map(async dao => {
      let owner = filter.owner;
      if (owner === undefined) {
        const completeAddresses = (await this.db.getCompleteAddresses()).find(address =>
          address.publicKey.equals(dao.publicKey),
        );
        if (completeAddresses === undefined) {
          throw new Error(`Cannot find complete address for public key ${dao.publicKey.toString()}`);
        }
        owner = completeAddresses.address;
      }
      return new ExtendedNote(dao.note, owner, dao.contractAddress, dao.storageSlot, dao.noteTypeId, dao.txHash);
    });
    return Promise.all(extendedNotes);
  }

  public async addNote(note: ExtendedNote) {
    const { publicKey } = (await this.db.getCompleteAddress(note.owner)) ?? {};
    if (!publicKey) {
      throw new Error('Unknown account.');
    }

    const nonces = await this.getNoteNonces(note);
    if (nonces.length === 0) {
      throw new Error(`Cannot find the note in tx: ${note.txHash}.`);
    }

    for (const nonce of nonces) {
      const { innerNoteHash, siloedNoteHash, uniqueSiloedNoteHash, innerNullifier } =
        await this.simulator.computeNoteHashAndNullifier(
          note.contractAddress,
          nonce,
          note.storageSlot,
          note.noteTypeId,
          note.note,
        );

      // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
      // This can always be `uniqueSiloedNoteHash` once notes added from public also include nonces.
      const noteHashToLookUp = nonce.isZero() ? siloedNoteHash : uniqueSiloedNoteHash;
      const index = await this.node.findLeafIndex('latest', MerkleTreeId.NOTE_HASH_TREE, noteHashToLookUp);
      if (index === undefined) {
        throw new Error('Note does not exist.');
      }

      const siloedNullifier = siloNullifier(note.contractAddress, innerNullifier!);
      const nullifierIndex = await this.node.findLeafIndex('latest', MerkleTreeId.NULLIFIER_TREE, siloedNullifier);
      if (nullifierIndex !== undefined) {
        throw new Error('The note has been destroyed.');
      }

      await this.db.addNote(
        new NoteDao(
          note.note,
          note.contractAddress,
          note.storageSlot,
          note.noteTypeId,
          note.txHash,
          nonce,
          innerNoteHash,
          siloedNullifier,
          index,
          publicKey,
        ),
      );
    }
  }

  /**
   * Finds the nonce(s) for a given note.
   * @param note - The note to find the nonces for.
   * @returns The nonces of the note.
   * @remarks More than a single nonce may be returned since there might be more than one nonce for a given note.
   * TODO(#4956): Un-expose this
   */
  public async getNoteNonces(note: ExtendedNote): Promise<Fr[]> {
    const tx = await this.node.getTxEffect(note.txHash);
    if (!tx) {
      throw new Error(`Unknown tx: ${note.txHash}`);
    }

    const nonces: Fr[] = [];
    const firstNullifier = tx.nullifiers[0];
    const hashes = tx.noteHashes;
    for (let i = 0; i < hashes.length; ++i) {
      const hash = hashes[i];
      if (hash.equals(Fr.ZERO)) {
        break;
      }

      const nonce = computeCommitmentNonce(firstNullifier, i);
      const { siloedNoteHash, uniqueSiloedNoteHash } = await this.simulator.computeNoteHashAndNullifier(
        note.contractAddress,
        nonce,
        note.storageSlot,
        note.noteTypeId,
        note.note,
      );
      // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
      // Remove this once notes added from public also include nonces.
      if (hash.equals(siloedNoteHash)) {
        nonces.push(Fr.ZERO);
        break;
      }
      if (hash.equals(uniqueSiloedNoteHash)) {
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

  public async proveTx(txRequest: TxExecutionRequest, simulatePublic: boolean) {
    return (await this.simulateTx(txRequest, simulatePublic)).tx;
  }

  public async simulateTx(
    txRequest: TxExecutionRequest,
    simulatePublic: boolean,
    msgSender: AztecAddress | undefined = undefined,
  ) {
    if (!txRequest.functionData.isPrivate) {
      throw new Error(`Public entrypoints are not allowed`);
    }
    return await this.jobQueue.put(async () => {
      const timer = new Timer();
      const simulatedTx = await this.#simulateAndProve(txRequest, msgSender);
      // We log only if the msgSender is undefined, as simulating with a different msgSender
      // is unlikely to be a real transaction, and likely to be only used to read data.
      // Meaning that it will not necessarily have produced a nullifier (and thus have no TxHash)
      // If we log, the `getTxHash` function will throw.

      if (!msgSender) {
        this.log(`Processed private part of ${simulatedTx.tx.getTxHash()}`, {
          eventName: 'tx-pxe-processing',
          duration: timer.ms(),
          ...simulatedTx.tx.getStats(),
        } satisfies TxPXEProcessingStats);
      }

      if (simulatePublic) {
        // Only one transaction, so we can take index 0.
        simulatedTx.publicReturnValues = (await this.#simulatePublicCalls(simulatedTx.tx))[0];
      }

      if (!msgSender) {
        this.log.info(`Executed local simulation for ${simulatedTx.tx.getTxHash()}`);
      }
      return simulatedTx;
    });
  }

  public async sendTx(tx: Tx): Promise<TxHash> {
    const txHash = tx.getTxHash();
    if (await this.node.getTxEffect(txHash)) {
      throw new Error(`A settled tx with equal hash ${txHash.toString()} exists.`);
    }
    this.log.info(`Sending transaction ${txHash}`);
    await this.node.sendTx(tx);
    return txHash;
  }

  public async viewTx(
    functionName: string,
    args: any[],
    to: AztecAddress,
    _from?: AztecAddress,
  ): Promise<DecodedReturn> {
    // all simulations must be serialized w.r.t. the synchronizer
    return await this.jobQueue.put(async () => {
      // TODO - Should check if `from` has the permission to call the view function.
      const functionCall = await this.#getFunctionCall(functionName, args, to);
      const executionResult = await this.#simulateUnconstrained(functionCall);

      // TODO - Return typed result based on the function artifact.
      return executionResult;
    });
  }

  public getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    return this.node.getTxReceipt(txHash);
  }

  public getTxEffect(txHash: TxHash): Promise<TxEffect | undefined> {
    return this.node.getTxEffect(txHash);
  }

  async getBlockNumber(): Promise<number> {
    return await this.node.getBlockNumber();
  }

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  public getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    return this.node.getUnencryptedLogs(filter);
  }

  async #getFunctionCall(functionName: string, args: any[], to: AztecAddress): Promise<FunctionCall> {
    const contract = await this.db.getContract(to);
    if (!contract) {
      throw new Error(
        `Unknown contract ${to}: add it to PXE Service by calling server.addContracts(...).\nSee docs for context: https://docs.aztec.network/developers/debugging/aztecnr-errors#unknown-contract-0x0-add-it-to-pxe-by-calling-serveraddcontracts`,
      );
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
      nodeVersion: this.nodeVersion,
      chainId,
      protocolVersion: version,
      l1ContractAddresses: contractAddresses,
    };
    return nodeInfo;
  }

  /**
   * Retrieves the simulation parameters required to run an ACIR simulation.
   * This includes the contract address, function artifact, portal contract address, and historical tree roots.
   *
   * @param execRequest - The transaction request object containing details of the contract call.
   * @returns An object containing the contract address, function artifact, portal contract address, and historical tree roots.
   */
  async #getSimulationParameters(execRequest: FunctionCall | TxExecutionRequest) {
    const contractAddress = (execRequest as FunctionCall).to ?? (execRequest as TxExecutionRequest).origin;
    const functionArtifact = await this.contractDataOracle.getFunctionArtifact(
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
      functionArtifact: {
        ...functionArtifact,
        debug,
      },
      portalContract,
    };
  }

  async #simulate(txRequest: TxExecutionRequest, msgSender?: AztecAddress): Promise<ExecutionResult> {
    // TODO - Pause syncing while simulating.

    const { contractAddress, functionArtifact, portalContract } = await this.#getSimulationParameters(txRequest);

    this.log('Executing simulator...');
    try {
      const result = await this.simulator.run(txRequest, functionArtifact, contractAddress, portalContract, msgSender);
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
   * @returns The simulation result containing the outputs of the unconstrained function.
   */
  async #simulateUnconstrained(execRequest: FunctionCall) {
    const { contractAddress, functionArtifact } = await this.#getSimulationParameters(execRequest);

    this.log('Executing unconstrained simulator...');
    try {
      const result = await this.simulator.runUnconstrained(execRequest, functionArtifact, contractAddress);
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
      return await this.node.simulatePublicCalls(tx);
    } catch (err) {
      // Try to fill in the noir call stack since the PXE may have access to the debug metadata
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
   * The function takes in a transaction request, simulates it, and then generates a kernel proof
   * using the simulation result. Finally, it creates a private
   * transaction object with the generated proof and public inputs. If a new contract address is provided,
   * the function will also include the new contract's public functions in the transaction object.
   *
   * @param txExecutionRequest - The transaction request to be simulated and proved.
   * @param signature - The ECDSA signature for the transaction request.
   * @param msgSender - (Optional) The message sender to use for the simulation.
   * @returns An object tract contains:
   * A private transaction object containing the proof, public inputs, and encrypted logs.
   * The return values of the private execution
   */
  async #simulateAndProve(txExecutionRequest: TxExecutionRequest, msgSender?: AztecAddress) {
    // Get values that allow us to reconstruct the block hash
    const executionResult = await this.#simulate(txExecutionRequest, msgSender);

    const kernelOracle = new KernelOracle(this.contractDataOracle, this.keyStore, this.node);
    const kernelProver = new KernelProver(kernelOracle);
    this.log(`Executing kernel prover...`);
    const { proof, publicInputs } = await kernelProver.prove(txExecutionRequest.toTxRequest(), executionResult);

    const encryptedLogs = new EncryptedTxL2Logs(collectEncryptedLogs(executionResult));
    const unencryptedLogs = new UnencryptedTxL2Logs(collectUnencryptedLogs(executionResult));
    const enqueuedPublicFunctions = collectEnqueuedPublicFunctionCalls(executionResult);

    // HACK(#1639): Manually patches the ordering of the public call stack
    // TODO(#757): Enforce proper ordering of enqueued public calls
    await this.patchPublicCallStackOrdering(publicInputs, enqueuedPublicFunctions);

    const tx = new Tx(publicInputs, proof, encryptedLogs, unencryptedLogs, enqueuedPublicFunctions);
    return new SimulatedTx(tx, [executionResult.returnValues]);
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
            const functionArtifact = contract.functions.find(f => FunctionSelector.fromString(selector).equals(f));
            if (functionArtifact) {
              err.enrichWithFunctionName(
                parsedContractAddress,
                FunctionSelector.fromNameAndParameters(functionArtifact),
                functionArtifact.name,
              );
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
    publicInputs: PrivateKernelTailCircuitPublicInputs,
    enqueuedPublicCalls: PublicCallRequest[],
  ) {
    if (!publicInputs.forPublic) {
      return;
    }

    const enqueuedPublicCallStackItems = await Promise.all(enqueuedPublicCalls.map(c => c.toCallRequest()));

    // Validate all items in enqueued public calls are in the kernel emitted stack
    const enqueuedRevertiblePublicCallStackItems = enqueuedPublicCallStackItems.filter(enqueued =>
      publicInputs.forPublic!.end.publicCallStack.find(item => item.equals(enqueued)),
    );

    const revertibleStackSize = arrayNonEmptyLength(publicInputs.forPublic.end.publicCallStack, item => item.isEmpty());

    if (enqueuedRevertiblePublicCallStackItems.length !== revertibleStackSize) {
      throw new Error(
        `Enqueued revertible public function calls and revertible public call stack do not match.\nEnqueued calls: ${enqueuedRevertiblePublicCallStackItems
          .map(h => h.hash.toString())
          .join(', ')}\nPublic call stack: ${publicInputs.forPublic.end.publicCallStack
          .map(i => i.toString())
          .join(', ')}`,
      );
    }

    // Override kernel output
    publicInputs.forPublic.end.publicCallStack = padArrayEnd(
      enqueuedRevertiblePublicCallStackItems,
      CallRequest.empty(),
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    );

    // Do the same for non-revertible

    const enqueuedNonRevertiblePublicCallStackItems = enqueuedPublicCallStackItems.filter(enqueued =>
      publicInputs.forPublic!.endNonRevertibleData.publicCallStack.find(item => item.equals(enqueued)),
    );

    const nonRevertibleStackSize = arrayNonEmptyLength(
      publicInputs.forPublic.endNonRevertibleData.publicCallStack,
      item => item.isEmpty(),
    );

    if (enqueuedNonRevertiblePublicCallStackItems.length !== nonRevertibleStackSize) {
      throw new Error(
        `Enqueued non-revertible public function calls and non-revertible public call stack do not match.\nEnqueued calls: ${enqueuedNonRevertiblePublicCallStackItems
          .map(h => h.hash.toString())
          .join(', ')}\nPublic call stack: ${publicInputs.forPublic.endNonRevertibleData.publicCallStack
          .map(i => i.toString())
          .join(', ')}`,
      );
    }

    // Override kernel output
    publicInputs.forPublic.endNonRevertibleData.publicCallStack = padArrayEnd(
      enqueuedNonRevertiblePublicCallStackItems,
      CallRequest.empty(),
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

  public getKeyStore() {
    return this.keyStore;
  }

  public async isContractClassPubliclyRegistered(id: Fr): Promise<boolean> {
    return !!(await this.node.getContractClass(id));
  }

  public async isContractPubliclyDeployed(address: AztecAddress): Promise<boolean> {
    return !!(await this.node.getContract(address));
  }
}
