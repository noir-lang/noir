import {
  type AuthWitness,
  type AztecNode,
  EncryptedL2Log,
  EncryptedL2NoteLog,
  Event,
  L1EventPayload,
  L1NotePayload,
  Note,
  type NoteStatus,
  PublicExecutionRequest,
  TaggedLog,
  type UnencryptedL2Log,
} from '@aztec/circuit-types';
import {
  CallContext,
  FunctionSelector,
  type Header,
  type KeyValidationRequest,
  PrivateContextInputs,
  type TxContext,
} from '@aztec/circuits.js';
import { Aes128 } from '@aztec/circuits.js/barretenberg';
import { computeUniqueNoteHash, siloNoteHash } from '@aztec/circuits.js/hash';
import {
  EventSelector,
  type FunctionAbi,
  type FunctionArtifact,
  type NoteSelector,
  countArgumentsSize,
} from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr, GrumpkinScalar, type Point } from '@aztec/foundation/fields';
import { applyStringFormatting, createDebugLogger } from '@aztec/foundation/log';

import { type NoteData, toACVMWitness } from '../acvm/index.js';
import { type PackedValuesCache } from '../common/packed_values_cache.js';
import { type DBOracle } from './db_oracle.js';
import { type ExecutionNoteCache } from './execution_note_cache.js';
import {
  CountedLog,
  CountedNoteLog,
  CountedPublicExecutionRequest,
  type ExecutionResult,
  type NoteAndSlot,
} from './execution_result.js';
import { pickNotes } from './pick_notes.js';
import { executePrivateFunction } from './private_execution.js';
import { ViewDataOracle } from './view_data_oracle.js';

/**
 * The execution context for a client tx simulation.
 */
export class ClientExecutionContext extends ViewDataOracle {
  /**
   * New notes created during this execution.
   * It's possible that a note in this list has been nullified (in the same or other executions) and doesn't exist in the ExecutionNoteCache and the final proof data.
   * But we still include those notes in the execution result because their commitments are still in the public inputs of this execution.
   * This information is only for references (currently used for tests), and is not used for any sort of constrains.
   * Users can also use this to get a clearer idea of what's happened during a simulation.
   */
  private newNotes: NoteAndSlot[] = [];
  /**
   * Notes from previous transactions that are returned to the oracle call `getNotes` during this execution.
   * The mapping maps from the unique siloed note hash to the index for notes created in private executions.
   * It maps from siloed note hash to the index for notes created by public functions.
   *
   * They are not part of the ExecutionNoteCache and being forwarded to nested contexts via `extend()`
   * because these notes are meant to be maintained on a per-call basis
   * They should act as references for the read requests output by an app circuit via public inputs.
   */
  private noteHashLeafIndexMap: Map<bigint, bigint> = new Map();
  private nullifiedNoteHashCounters: Map<number, number> = new Map();
  private noteEncryptedLogs: CountedNoteLog[] = [];
  private encryptedLogs: CountedLog<EncryptedL2Log>[] = [];
  private unencryptedLogs: CountedLog<UnencryptedL2Log>[] = [];
  private nestedExecutions: ExecutionResult[] = [];
  private enqueuedPublicFunctionCalls: CountedPublicExecutionRequest[] = [];
  private publicTeardownFunctionCall: PublicExecutionRequest = PublicExecutionRequest.empty();

  constructor(
    contractAddress: AztecAddress,
    private readonly argsHash: Fr,
    private readonly txContext: TxContext,
    private readonly callContext: CallContext,
    /** Header of a block whose state is used during private execution (not the block the transaction is included in). */
    protected readonly historicalHeader: Header,
    /** List of transient auth witnesses to be used during this simulation */
    authWitnesses: AuthWitness[],
    private readonly packedValuesCache: PackedValuesCache,
    private readonly noteCache: ExecutionNoteCache,
    db: DBOracle,
    private node: AztecNode,
    protected sideEffectCounter: number = 0,
    log = createDebugLogger('aztec:simulator:client_execution_context'),
  ) {
    super(contractAddress, authWitnesses, db, node, log);
  }

  // We still need this function until we can get user-defined ordering of structs for fn arguments
  // TODO When that is sorted out on noir side, we can use instead the utilities in serialize.ts
  /**
   * Writes the function inputs to the initial witness.
   * @param abi - The function ABI.
   * @returns The initial witness.
   */
  public getInitialWitness(abi: FunctionAbi) {
    const argumentsSize = countArgumentsSize(abi);

    const args = this.packedValuesCache.unpack(this.argsHash);

    if (args.length !== argumentsSize) {
      throw new Error('Invalid arguments size');
    }

    const privateContextInputs = new PrivateContextInputs(
      this.callContext,
      this.historicalHeader,
      this.txContext,
      this.sideEffectCounter,
    );

    const fields = [...privateContextInputs.toFields(), ...args];
    return toACVMWitness(0, fields);
  }

  /**
   * The KernelProver will use this to fully populate witnesses and provide hints to the kernel circuit
   * regarding which note hash each settled read request corresponds to.
   */
  public getNoteHashLeafIndexMap() {
    return this.noteHashLeafIndexMap;
  }

  /**
   * Get the data for the newly created notes.
   * @param innerNoteHashes - Inner note hashes for the notes.
   */
  public getNewNotes(): NoteAndSlot[] {
    return this.newNotes;
  }

  public getNullifiedNoteHashCounters() {
    return this.nullifiedNoteHashCounters;
  }

  /**
   * Return the note encrypted logs emitted during this execution.
   */
  public getNoteEncryptedLogs() {
    return this.noteEncryptedLogs;
  }

  /**
   * Return the encrypted logs emitted during this execution.
   */
  public getEncryptedLogs() {
    return this.encryptedLogs;
  }

  /**
   * Return the encrypted logs emitted during this execution.
   */
  public getUnencryptedLogs() {
    return this.unencryptedLogs;
  }

  /**
   * Return the nested execution results during this execution.
   */
  public getNestedExecutions() {
    return this.nestedExecutions;
  }

  /**
   * Return the enqueued public function calls during this execution.
   */
  public getEnqueuedPublicFunctionCalls() {
    return this.enqueuedPublicFunctionCalls;
  }

  /**
   * Return the public teardown function call set during this execution.
   */
  public getPublicTeardownFunctionCall() {
    return this.publicTeardownFunctionCall;
  }

  /**
   * Pack the given array of arguments.
   * @param args - Arguments to pack
   */
  public override packArgumentsArray(args: Fr[]): Promise<Fr> {
    return Promise.resolve(this.packedValuesCache.pack(args));
  }

  /**
   * Pack the given returns.
   * @param returns - Returns to pack
   */
  public override packReturns(returns: Fr[]): Promise<Fr> {
    return Promise.resolve(this.packedValuesCache.pack(returns));
  }

  /**
   * Unpack the given returns.
   * @param returnsHash - Returns hash to unpack
   */
  public override unpackReturns(returnsHash: Fr): Promise<Fr[]> {
    return Promise.resolve(this.packedValuesCache.unpack(returnsHash));
  }

  /**
   * Gets some notes for a storage slot.
   *
   * @remarks
   * Check for pending notes with matching slot.
   * Real notes coming from DB will have a leafIndex which
   * represents their index in the note hash tree.
   *
   * @param storageSlot - The storage slot.
   * @param numSelects - The number of valid selects in selectBy and selectValues.
   * @param selectBy - An array of indices of the fields to selects.
   * @param selectValues - The values to match.
   * @param selectComparators - The comparators to match by.
   * @param sortBy - An array of indices of the fields to sort.
   * @param sortOrder - The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * @param limit - The number of notes to retrieve per query.
   * @param offset - The starting index for pagination.
   * @param status - The status of notes to fetch.
   * @returns Array of note data.
   */
  public override async getNotes(
    storageSlot: Fr,
    numSelects: number,
    selectByIndexes: number[],
    selectByOffsets: number[],
    selectByLengths: number[],
    selectValues: Fr[],
    selectComparators: number[],
    sortByIndexes: number[],
    sortByOffsets: number[],
    sortByLengths: number[],
    sortOrder: number[],
    limit: number,
    offset: number,
    status: NoteStatus,
  ): Promise<NoteData[]> {
    // Nullified pending notes are already removed from the list.
    const pendingNotes = this.noteCache.getNotes(this.callContext.storageContractAddress, storageSlot);

    const pendingNullifiers = this.noteCache.getNullifiers(this.callContext.storageContractAddress);
    const dbNotes = await this.db.getNotes(this.callContext.storageContractAddress, storageSlot, status);
    const dbNotesFiltered = dbNotes.filter(n => !pendingNullifiers.has((n.siloedNullifier as Fr).value));

    const notes = pickNotes<NoteData>([...dbNotesFiltered, ...pendingNotes], {
      selects: selectByIndexes.slice(0, numSelects).map((index, i) => ({
        selector: { index, offset: selectByOffsets[i], length: selectByLengths[i] },
        value: selectValues[i],
        comparator: selectComparators[i],
      })),
      sorts: sortByIndexes.map((index, i) => ({
        selector: { index, offset: sortByOffsets[i], length: sortByLengths[i] },
        order: sortOrder[i],
      })),
      limit,
      offset,
    });

    this.log.debug(
      `Returning ${notes.length} notes for ${this.callContext.storageContractAddress} at ${storageSlot}: ${notes
        .map(n => `${n.nonce.toString()}:[${n.note.items.map(i => i.toString()).join(',')}]`)
        .join(', ')}`,
    );

    notes.forEach(n => {
      if (n.index !== undefined) {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
        // Should always call computeUniqueNoteHash when publicly created notes include nonces.
        const uniqueNoteHash = n.nonce.isZero() ? n.innerNoteHash : computeUniqueNoteHash(n.nonce, n.innerNoteHash);
        const siloedNoteHash = siloNoteHash(n.contractAddress, uniqueNoteHash);
        const noteHashForReadRequest = siloedNoteHash;
        this.noteHashLeafIndexMap.set(noteHashForReadRequest.toBigInt(), n.index);
      }
    });

    return notes;
  }

  /**
   * Keep track of the new note created during execution.
   * It can be used in subsequent calls (or transactions when chaining txs is possible).
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param noteTypeId - The type ID of the note.
   * @param noteItems - The items to be included in a Note.
   * @param innerNoteHash - The inner note hash of the new note.
   * @returns
   */
  public override notifyCreatedNote(
    storageSlot: Fr,
    noteTypeId: NoteSelector,
    noteItems: Fr[],
    innerNoteHash: Fr,
    counter: number,
  ) {
    const note = new Note(noteItems);
    this.noteCache.addNewNote(
      {
        contractAddress: this.callContext.storageContractAddress,
        storageSlot,
        nonce: Fr.ZERO, // Nonce cannot be known during private execution.
        note,
        siloedNullifier: undefined, // Siloed nullifier cannot be known for newly created note.
        innerNoteHash,
      },
      counter,
    );
    this.newNotes.push({
      storageSlot,
      noteTypeId,
      note,
    });
  }

  /**
   * Adding a siloed nullifier into the current set of all pending nullifiers created
   * within the current transaction/execution.
   * @param innerNullifier - The pending nullifier to add in the list (not yet siloed by contract address).
   * @param innerNoteHash - The inner note hash of the new note.
   */
  public override notifyNullifiedNote(innerNullifier: Fr, innerNoteHash: Fr, counter: number) {
    const nullifiedNoteHashCounter = this.noteCache.nullifyNote(
      this.callContext.storageContractAddress,
      innerNullifier,
      innerNoteHash,
    );
    if (nullifiedNoteHashCounter !== undefined) {
      this.nullifiedNoteHashCounters.set(nullifiedNoteHashCounter, counter);
    }
    return Promise.resolve();
  }

  /**
   * Emit encrypted data
   * @param contractAddress - The contract emitting the encrypted event.
   * @param randomness - A value used to mask the contract address we are siloing with.
   * @param encryptedEvent - The encrypted event data.
   * @param counter - The effects counter.
   */
  public override emitEncryptedEventLog(
    contractAddress: AztecAddress,
    randomness: Fr,
    encryptedEvent: Buffer,
    counter: number,
  ) {
    // In some cases, we actually want to reveal the contract address we are siloing with:
    // e.g. 'handshaking' contract w/ known address
    // An app providing randomness = 0 signals to not mask the address.
    const maskedContractAddress = randomness.isZero()
      ? contractAddress.toField()
      : pedersenHash([contractAddress, randomness], 0);
    const encryptedLog = new CountedLog(new EncryptedL2Log(encryptedEvent, maskedContractAddress), counter);
    this.encryptedLogs.push(encryptedLog);
  }

  /**
   * Emit encrypted note data
   * @param noteHashCounter - The note hash counter.
   * @param encryptedNote - The encrypted note data.
   * @param counter - The log counter.
   */
  public override emitEncryptedNoteLog(noteHashCounter: number, encryptedNote: Buffer, counter: number) {
    const encryptedLog = new CountedNoteLog(new EncryptedL2NoteLog(encryptedNote), counter, noteHashCounter);
    this.noteEncryptedLogs.push(encryptedLog);
  }

  /**
   * Encrypt an event
   * @param contractAddress - The contract emitting the encrypted event.
   * @param randomness - A value used to mask the contract address we are siloing with.
   * @param eventTypeId - The type ID of the event (function selector).
   * @param ovKeys - The outgoing viewing keys to use to encrypt.
   * @param ivpkM - The master incoming viewing public key.
   * @param recipient - The recipient of the encrypted event log.
   * @param preimage - The event preimage.
   */
  public override computeEncryptedEventLog(
    contractAddress: AztecAddress,
    randomness: Fr,
    eventTypeId: Fr,
    ovKeys: KeyValidationRequest,
    ivpkM: Point,
    recipient: AztecAddress,
    preimage: Fr[],
  ) {
    const event = new Event(preimage);
    const l1EventPayload = new L1EventPayload(event, contractAddress, randomness, EventSelector.fromField(eventTypeId));
    const taggedEvent = new TaggedLog(l1EventPayload);

    const ephSk = GrumpkinScalar.random();

    return taggedEvent.encrypt(ephSk, recipient, ivpkM, ovKeys);
  }

  /**
   * Encrypt a note
   * @param contractAddress - The contract address of the note.
   * @param storageSlot - The storage slot the note is at.
   * @param noteTypeId - The type ID of the note.
   * @param ovKeys - The outgoing viewing keys to use to encrypt.
   * @param ivpkM - The master incoming viewing public key.
   * @param recipient - The recipient of the encrypted note log.
   * @param preimage - The note preimage.
   */
  public override computeEncryptedNoteLog(
    contractAddress: AztecAddress,
    storageSlot: Fr,
    noteTypeId: NoteSelector,
    ovKeys: KeyValidationRequest,
    ivpkM: Point,
    recipient: AztecAddress,
    preimage: Fr[],
  ) {
    const note = new Note(preimage);
    const l1NotePayload = new L1NotePayload(note, contractAddress, storageSlot, noteTypeId);
    const taggedNote = new TaggedLog(l1NotePayload);

    const ephSk = GrumpkinScalar.random();

    return taggedNote.encrypt(ephSk, recipient, ivpkM, ovKeys);
  }

  /**
   * Emit an unencrypted log.
   * @param log - The unencrypted log to be emitted.
   */
  public override emitUnencryptedLog(log: UnencryptedL2Log, counter: number) {
    this.unencryptedLogs.push(new CountedLog(log, counter));
    const text = log.toHumanReadable();
    this.log.verbose(`Emitted unencrypted log: "${text.length > 100 ? text.slice(0, 100) + '...' : text}"`);
  }

  /**
   * Emit a contract class unencrypted log.
   * This fn exists separately from emitUnencryptedLog because sha hashing the preimage
   * is too large to compile (16,200 fields, 518,400 bytes) => the oracle hashes it.
   * See private_context.nr
   * @param log - The unencrypted log to be emitted.
   */
  public override emitContractClassUnencryptedLog(log: UnencryptedL2Log, counter: number) {
    this.unencryptedLogs.push(new CountedLog(log, counter));
    const text = log.toHumanReadable();
    this.log.verbose(
      `Emitted unencrypted log from ContractClassRegisterer: "${
        text.length > 100 ? text.slice(0, 100) + '...' : text
      }"`,
    );
    return Fr.fromBuffer(log.hash());
  }

  #checkValidStaticCall(childExecutionResult: ExecutionResult) {
    if (
      childExecutionResult.callStackItem.publicInputs.noteHashes.some(item => !item.isEmpty()) ||
      childExecutionResult.callStackItem.publicInputs.nullifiers.some(item => !item.isEmpty()) ||
      childExecutionResult.callStackItem.publicInputs.l2ToL1Msgs.some(item => !item.isEmpty()) ||
      childExecutionResult.callStackItem.publicInputs.encryptedLogsHashes.some(item => !item.isEmpty()) ||
      childExecutionResult.callStackItem.publicInputs.unencryptedLogsHashes.some(item => !item.isEmpty())
    ) {
      throw new Error(`Static call cannot update the state, emit L2->L1 messages or generate logs`);
    }
  }

  /**
   * Calls a private function as a nested execution.
   * @param targetContractAddress - The address of the contract to call.
   * @param functionSelector - The function selector of the function to call.
   * @param argsHash - The packed arguments to pass to the function.
   * @param sideEffectCounter - The side effect counter at the start of the call.
   * @param isStaticCall - Whether the call is a static call.
   * @param isDelegateCall - Whether the call is a delegate call.
   * @returns The execution result.
   */
  override async callPrivateFunction(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    this.log.debug(
      `Calling private function ${this.contractAddress}:${functionSelector} from ${this.callContext.storageContractAddress}`,
    );

    isStaticCall = isStaticCall || this.callContext.isStaticCall;

    const targetArtifact = await this.db.getFunctionArtifact(targetContractAddress, functionSelector);

    const derivedTxContext = this.txContext.clone();

    const derivedCallContext = this.deriveCallContext(
      targetContractAddress,
      targetArtifact,
      isDelegateCall,
      isStaticCall,
    );

    const context = new ClientExecutionContext(
      targetContractAddress,
      argsHash,
      derivedTxContext,
      derivedCallContext,
      this.historicalHeader,
      this.authWitnesses,
      this.packedValuesCache,
      this.noteCache,
      this.db,
      this.node,
      sideEffectCounter,
    );

    const childExecutionResult = await executePrivateFunction(
      context,
      targetArtifact,
      targetContractAddress,
      functionSelector,
    );

    if (isStaticCall) {
      this.#checkValidStaticCall(childExecutionResult);
    }

    this.nestedExecutions.push(childExecutionResult);

    const publicInputs = childExecutionResult.callStackItem.publicInputs;
    return {
      endSideEffectCounter: publicInputs.endSideEffectCounter,
      returnsHash: publicInputs.returnsHash,
    };
  }

  /**
   * Creates a PublicCallStackItem object representing the request to call a public function.
   * @param targetContractAddress - The address of the contract to call.
   * @param functionSelector - The function selector of the function to call.
   * @param argsHash - The packed arguments to pass to the function.
   * @param sideEffectCounter - The side effect counter at the start of the call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The public call stack item with the request information.
   */
  protected async createPublicExecutionRequest(
    callType: 'enqueued' | 'teardown',
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    const targetArtifact = await this.db.getFunctionArtifact(targetContractAddress, functionSelector);
    const derivedCallContext = this.deriveCallContext(
      targetContractAddress,
      targetArtifact,
      isDelegateCall,
      isStaticCall,
    );
    const args = this.packedValuesCache.unpack(argsHash);

    this.log.verbose(
      `Created PublicExecutionRequest of type [${callType}], side-effect counter [${sideEffectCounter}] to ${targetContractAddress}:${functionSelector}(${targetArtifact.name})`,
    );

    const request = PublicExecutionRequest.from({
      args,
      callContext: derivedCallContext,
      contractAddress: targetContractAddress,
    });

    if (callType === 'enqueued') {
      this.enqueuedPublicFunctionCalls.push(new CountedPublicExecutionRequest(request, sideEffectCounter));
    } else {
      this.publicTeardownFunctionCall = request;
    }
  }

  /**
   * Creates and enqueues a PublicCallStackItem object representing the request to call a public function. No function
   * is actually called, since that must happen on the sequencer side. All the fields related to the result
   * of the execution are empty.
   * @param targetContractAddress - The address of the contract to call.
   * @param functionSelector - The function selector of the function to call.
   * @param argsHash - The packed arguments to pass to the function.
   * @param sideEffectCounter - The side effect counter at the start of the call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The public call stack item with the request information.
   */
  public override async enqueuePublicFunctionCall(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    await this.createPublicExecutionRequest(
      'enqueued',
      targetContractAddress,
      functionSelector,
      argsHash,
      sideEffectCounter,
      isStaticCall,
      isDelegateCall,
    );
  }

  /**
   * Creates a PublicCallStackItem and sets it as the public teardown function. No function
   * is actually called, since that must happen on the sequencer side. All the fields related to the result
   * of the execution are empty.
   * @param targetContractAddress - The address of the contract to call.
   * @param functionSelector - The function selector of the function to call.
   * @param argsHash - The packed arguments to pass to the function.
   * @param sideEffectCounter - The side effect counter at the start of the call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The public call stack item with the request information.
   */
  public override async setPublicTeardownFunctionCall(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    await this.createPublicExecutionRequest(
      'teardown',
      targetContractAddress,
      functionSelector,
      argsHash,
      sideEffectCounter,
      isStaticCall,
      isDelegateCall,
    );
  }

  /**
   * Derives the call context for a nested execution.
   * @param targetContractAddress - The address of the contract being called.
   * @param targetArtifact - The artifact of the function being called.
   * @param isDelegateCall - Whether the call is a delegate call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The derived call context.
   */
  private deriveCallContext(
    targetContractAddress: AztecAddress,
    targetArtifact: FunctionArtifact,
    isDelegateCall = false,
    isStaticCall = false,
  ) {
    return new CallContext(
      isDelegateCall ? this.callContext.msgSender : this.contractAddress,
      isDelegateCall ? this.contractAddress : targetContractAddress,
      FunctionSelector.fromNameAndParameters(targetArtifact.name, targetArtifact.parameters),
      isDelegateCall,
      isStaticCall,
    );
  }

  /**
   * Read the public storage data.
   * @param contractAddress - The address to read storage from.
   * @param startStorageSlot - The starting storage slot.
   * @param blockNumber - The block number to read storage at.
   * @param numberOfElements - Number of elements to read from the starting storage slot.
   */
  public override async storageRead(
    contractAddress: Fr,
    startStorageSlot: Fr,
    blockNumber: number,
    numberOfElements: number,
  ): Promise<Fr[]> {
    const values = [];
    for (let i = 0n; i < numberOfElements; i++) {
      const storageSlot = new Fr(startStorageSlot.value + i);

      const value = await this.aztecNode.getPublicStorageAt(contractAddress, storageSlot, blockNumber);
      this.log.debug(
        `Oracle storage read: slot=${storageSlot.toString()} address-${contractAddress.toString()} value=${value}`,
      );

      values.push(value);
    }
    return values;
  }

  public override aes128Encrypt(input: Buffer, initializationVector: Buffer, key: Buffer): Buffer {
    const aes128 = new Aes128();
    return aes128.encryptBufferCBC(input, initializationVector, key);
  }

  public override debugLog(message: string, fields: Fr[]) {
    this.log.verbose(`debug_log ${applyStringFormatting(message, fields)}`);
  }

  public getDebugFunctionName() {
    return this.db.getDebugFunctionName(this.contractAddress, this.callContext.functionSelector);
  }
}
