import {
  type AuthWitness,
  type AztecNode,
  EncryptedNoteTxL2Logs,
  EncryptedTxL2Logs,
  ExtendedNote,
  type FunctionCall,
  type GetUnencryptedLogsResponse,
  type L2Block,
  type LogFilter,
  MerkleTreeId,
  type NoteFilter,
  type PXE,
  type PXEInfo,
  type ProofCreator,
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
import {
  AztecAddress,
  type CompleteAddress,
  type PartialAddress,
  computeContractClassId,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import { computeNoteHashNonce, siloNullifier } from '@aztec/circuits.js/hash';
import { type ContractArtifact, type DecodedReturn, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { type Fq, Fr } from '@aztec/foundation/fields';
import { SerialQueue } from '@aztec/foundation/fifo';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { type KeyStore } from '@aztec/key-store';
import { getCanonicalClassRegistererAddress } from '@aztec/protocol-contracts/class-registerer';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalInstanceDeployer } from '@aztec/protocol-contracts/instance-deployer';
import { getCanonicalKeyRegistryAddress } from '@aztec/protocol-contracts/key-registry';
import { getCanonicalMultiCallEntrypointAddress } from '@aztec/protocol-contracts/multi-call-entrypoint';
import {
  type AcirSimulator,
  type ExecutionResult,
  accumulateReturnValues,
  collectEnqueuedPublicFunctionCalls,
  collectPublicTeardownFunctionCall,
  collectSortedEncryptedLogs,
  collectSortedNoteEncryptedLogs,
  collectSortedUnencryptedLogs,
  resolveOpcodeLocations,
} from '@aztec/simulator';
import { type ContractClassWithId, type ContractInstanceWithAddress } from '@aztec/types/contracts';
import { type NodeInfo } from '@aztec/types/interfaces';

import { type PXEServiceConfig, getPackageInfo } from '../config/index.js';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { IncomingNoteDao } from '../database/incoming_note_dao.js';
import { type PxeDatabase } from '../database/index.js';
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
  private packageVersion: string;
  // serialize synchronizer and calls to proveTx.
  // ensures that state is not changed while simulating
  private jobQueue = new SerialQueue();

  constructor(
    private keyStore: KeyStore,
    private node: AztecNode,
    private db: PxeDatabase,
    private proofCreator: ProofCreator,
    private config: PXEServiceConfig,
    logSuffix?: string,
  ) {
    this.log = createDebugLogger(logSuffix ? `aztec:pxe_service_${logSuffix}` : `aztec:pxe_service`);
    this.synchronizer = new Synchronizer(node, db, this.jobQueue, logSuffix);
    this.contractDataOracle = new ContractDataOracle(db);
    this.simulator = getAcirSimulator(db, node, keyStore, this.contractDataOracle);
    this.packageVersion = getPackageInfo().version;

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
    const accounts = await this.keyStore.getAccounts();
    const publicKeys = accounts.map(async account => await this.keyStore.getMasterIncomingViewingPublicKey(account));
    const publicKeysSet = new Set(publicKeys.map(k => k.toString()));

    const registeredAddresses = await this.db.getCompleteAddresses();

    let count = 0;
    for (const address of registeredAddresses) {
      if (!publicKeysSet.has(address.publicKeys.masterIncomingViewingPublicKey.toString())) {
        continue;
      }

      count++;
      await this.synchronizer.addAccount(address.address, this.keyStore, this.config.l2StartingBlock);
    }

    if (count > 0) {
      this.log.info(`Restored ${count} accounts`);
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

  async rotateNskM(account: AztecAddress, secretKey: Fq): Promise<void> {
    await this.keyStore.rotateMasterNullifierKey(account, secretKey);
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

  public async registerAccount(secretKey: Fr, partialAddress: PartialAddress): Promise<CompleteAddress> {
    const accounts = await this.keyStore.getAccounts();
    const accountCompleteAddress = await this.keyStore.addAccount(secretKey, partialAddress);
    if (accounts.includes(accountCompleteAddress.address)) {
      this.log.info(`Account:\n "${accountCompleteAddress.address.toString()}"\n already registered.`);
      return accountCompleteAddress;
    } else {
      await this.synchronizer.addAccount(accountCompleteAddress.address, this.keyStore, this.config.l2StartingBlock);
      this.log.info(`Registered account ${accountCompleteAddress.address.toString()}`);
      this.log.debug(`Registered account\n ${accountCompleteAddress.toReadableString()}`);
    }

    await this.db.addCompleteAddress(accountCompleteAddress);
    return accountCompleteAddress;
  }

  public async getRegisteredAccounts(): Promise<CompleteAddress[]> {
    // Get complete addresses of both the recipients and the accounts
    const completeAddresses = await this.db.getCompleteAddresses();
    // Filter out the addresses not corresponding to accounts
    const accounts = await this.keyStore.getAccounts();
    return completeAddresses.filter(completeAddress =>
      accounts.find(address => address.equals(completeAddress.address)),
    );
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
    const completeAddresses = await this.db.getCompleteAddresses();
    // Filter out the addresses corresponding to accounts
    const accounts = await this.keyStore.getAccounts();
    const recipients = completeAddresses.filter(
      completeAddress => !accounts.find(account => account.equals(completeAddress.address)),
    );
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
          address.publicKeys.masterIncomingViewingPublicKey.equals(dao.ivpkM),
        );
        if (completeAddresses === undefined) {
          throw new Error(`Cannot find complete address for IvpkM ${dao.ivpkM.toString()}`);
        }
        owner = completeAddresses.address;
      }
      return new ExtendedNote(dao.note, owner, dao.contractAddress, dao.storageSlot, dao.noteTypeId, dao.txHash);
    });
    return Promise.all(extendedNotes);
  }

  public async addNote(note: ExtendedNote) {
    const owner = await this.db.getCompleteAddress(note.owner);
    if (!owner) {
      throw new Error(`Unknown account: ${note.owner.toString()}`);
    }

    const nonces = await this.getNoteNonces(note);
    if (nonces.length === 0) {
      throw new Error(`Cannot find the note in tx: ${note.txHash}.`);
    }

    for (const nonce of nonces) {
      const { innerNoteHash, siloedNoteHash, innerNullifier } = await this.simulator.computeNoteHashAndNullifier(
        note.contractAddress,
        nonce,
        note.storageSlot,
        note.noteTypeId,
        note.note,
      );

      const index = await this.node.findLeafIndex('latest', MerkleTreeId.NOTE_HASH_TREE, siloedNoteHash);
      if (index === undefined) {
        throw new Error('Note does not exist.');
      }

      const siloedNullifier = siloNullifier(note.contractAddress, innerNullifier!);
      const nullifierIndex = await this.node.findLeafIndex('latest', MerkleTreeId.NULLIFIER_TREE, siloedNullifier);
      if (nullifierIndex !== undefined) {
        throw new Error('The note has been destroyed.');
      }

      await this.db.addNote(
        new IncomingNoteDao(
          note.note,
          note.contractAddress,
          note.storageSlot,
          note.noteTypeId,
          note.txHash,
          nonce,
          innerNoteHash,
          siloedNullifier,
          index,
          owner.publicKeys.masterIncomingViewingPublicKey,
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

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
    // Remove this once notes added from public also include nonces.
    {
      const publicNoteNonce = Fr.ZERO;
      const { siloedNoteHash } = await this.simulator.computeNoteHashAndNullifier(
        note.contractAddress,
        publicNoteNonce,
        note.storageSlot,
        note.noteTypeId,
        note.note,
      );
      if (tx.noteHashes.some(hash => hash.equals(siloedNoteHash))) {
        nonces.push(publicNoteNonce);
      }
    }

    const firstNullifier = tx.nullifiers[0];
    const hashes = tx.noteHashes;
    for (let i = 0; i < hashes.length; ++i) {
      const hash = hashes[i];
      if (hash.equals(Fr.ZERO)) {
        break;
      }

      const nonce = computeNoteHashNonce(firstNullifier, i);
      const { siloedNoteHash } = await this.simulator.computeNoteHashAndNullifier(
        note.contractAddress,
        nonce,
        note.storageSlot,
        note.noteTypeId,
        note.note,
      );
      if (hash.equals(siloedNoteHash)) {
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
  ): Promise<SimulatedTx> {
    return await this.jobQueue.put(async () => {
      const simulatedTx = await this.#simulateAndProve(txRequest, msgSender);
      // We log only if the msgSender is undefined, as simulating with a different msgSender
      // is unlikely to be a real transaction, and likely to be only used to read data.
      // Meaning that it will not necessarily have produced a nullifier (and thus have no TxHash)
      // If we log, the `getTxHash` function will throw.

      if (simulatePublic) {
        simulatedTx.publicOutput = await this.#simulatePublicCalls(simulatedTx.tx);
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

  public async simulateUnconstrained(
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
      name: functionDao.name,
      args: encodeArguments(functionDao, args),
      selector: FunctionSelector.fromNameAndParameters(functionDao.name, functionDao.parameters),
      type: functionDao.functionType,
      to,
      isStatic: functionDao.isStatic,
      returnTypes: functionDao.returnTypes,
    };
  }

  public async getNodeInfo(): Promise<NodeInfo> {
    const [nodeVersion, protocolVersion, chainId, contractAddresses, protocolContractAddresses] = await Promise.all([
      this.node.getNodeVersion(),
      this.node.getVersion(),
      this.node.getChainId(),
      this.node.getL1ContractAddresses(),
      this.node.getProtocolContractAddresses(),
    ]);

    const nodeInfo: NodeInfo = {
      nodeVersion,
      chainId,
      protocolVersion,
      l1ContractAddresses: contractAddresses,
      protocolContractAddresses: protocolContractAddresses,
    };

    return nodeInfo;
  }

  public getPXEInfo(): Promise<PXEInfo> {
    return Promise.resolve({
      pxeVersion: this.packageVersion,
      protocolContractAddresses: {
        classRegisterer: getCanonicalClassRegistererAddress(),
        gasToken: getCanonicalGasToken().address,
        instanceDeployer: getCanonicalInstanceDeployer().address,
        keyRegistry: getCanonicalKeyRegistryAddress(),
        multiCallEntrypoint: getCanonicalMultiCallEntrypointAddress(),
      },
    });
  }

  /**
   * Retrieves the simulation parameters required to run an ACIR simulation.
   * This includes the contract address, function artifact, and historical tree roots.
   *
   * @param execRequest - The transaction request object containing details of the contract call.
   * @returns An object containing the contract address, function artifact, and historical tree roots.
   */
  async #getSimulationParameters(execRequest: FunctionCall | TxExecutionRequest) {
    const contractAddress = (execRequest as FunctionCall).to ?? (execRequest as TxExecutionRequest).origin;
    const functionSelector =
      (execRequest as FunctionCall).selector ?? (execRequest as TxExecutionRequest).functionSelector;
    const functionArtifact = await this.contractDataOracle.getFunctionArtifact(contractAddress, functionSelector);
    const debug = await this.contractDataOracle.getFunctionDebugMetadata(contractAddress, functionSelector);

    return {
      contractAddress,
      functionArtifact: {
        ...functionArtifact,
        debug,
      },
    };
  }

  async #simulate(txRequest: TxExecutionRequest, msgSender?: AztecAddress): Promise<ExecutionResult> {
    // TODO - Pause syncing while simulating.

    const { contractAddress, functionArtifact } = await this.#getSimulationParameters(txRequest);

    this.log.debug('Executing simulator...');
    try {
      const result = await this.simulator.run(txRequest, functionArtifact, contractAddress, msgSender);
      this.log.verbose(`Simulation completed for ${contractAddress.toString()}:${functionArtifact.name}`);
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

    this.log.debug('Executing unconstrained simulator...');
    try {
      const result = await this.simulator.runUnconstrained(execRequest, functionArtifact, contractAddress);
      this.log.verbose(`Unconstrained simulation for ${contractAddress}.${functionArtifact.name} completed`);

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
    const kernelProver = new KernelProver(kernelOracle, this.proofCreator);
    this.log.debug(`Executing kernel prover...`);
    const { proof, publicInputs } = await kernelProver.prove(txExecutionRequest.toTxRequest(), executionResult);

    const noteEncryptedLogs = new EncryptedNoteTxL2Logs([collectSortedNoteEncryptedLogs(executionResult)]);
    const unencryptedLogs = new UnencryptedTxL2Logs([collectSortedUnencryptedLogs(executionResult)]);
    const encryptedLogs = new EncryptedTxL2Logs([collectSortedEncryptedLogs(executionResult)]);
    const enqueuedPublicFunctions = collectEnqueuedPublicFunctionCalls(executionResult);
    const teardownPublicFunction = collectPublicTeardownFunctionCall(executionResult);

    const tx = new Tx(
      publicInputs,
      proof.binaryProof,
      noteEncryptedLogs,
      encryptedLogs,
      unencryptedLogs,
      enqueuedPublicFunctions,
      teardownPublicFunction,
    );

    return new SimulatedTx(tx, accumulateReturnValues(executionResult));
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

  public async isGlobalStateSynchronized() {
    return await this.synchronizer.isGlobalStateSynchronized();
  }

  public async isAccountStateSynchronized(account: AztecAddress) {
    return await this.synchronizer.isAccountStateSynchronized(account);
  }

  public getSyncStatus() {
    return Promise.resolve(this.synchronizer.getSyncStatus());
  }

  public getSyncStats() {
    return Promise.resolve(this.synchronizer.getSyncStats());
  }

  public async isContractClassPubliclyRegistered(id: Fr): Promise<boolean> {
    return !!(await this.node.getContractClass(id));
  }

  public async isContractPubliclyDeployed(address: AztecAddress): Promise<boolean> {
    return !!(await this.node.getContract(address));
  }
}
