import {
  type AuthWitness,
  type AztecNode,
  EncryptedFunctionL2Logs,
  EncryptedL2Log,
  L1NotePayload,
  Note,
  type NoteStatus,
  TaggedNote,
  UnencryptedFunctionL2Logs,
  type UnencryptedL2Log,
} from '@aztec/circuit-types';
import {
  CallContext,
  FunctionData,
  FunctionSelector,
  type Header,
  NoteHashReadRequestMembershipWitness,
  PublicCallRequest,
  type SideEffect,
  TxContext,
} from '@aztec/circuits.js';
import { type Grumpkin } from '@aztec/circuits.js/barretenberg';
import { computePublicDataTreeLeafSlot, computeUniqueCommitment, siloNoteHash } from '@aztec/circuits.js/hash';
import { type FunctionAbi, type FunctionArtifact, countArgumentsSize } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, type Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import { type NoteData, toACVMWitness } from '../acvm/index.js';
import { type PackedArgsCache } from '../common/packed_args_cache.js';
import { type DBOracle } from './db_oracle.js';
import { type ExecutionNoteCache } from './execution_note_cache.js';
import { type ExecutionResult, type NoteAndSlot } from './execution_result.js';
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
  private gotNotes: Map<bigint, bigint> = new Map();
  private encryptedLogs: EncryptedL2Log[] = [];
  private unencryptedLogs: UnencryptedL2Log[] = [];
  private nestedExecutions: ExecutionResult[] = [];
  private enqueuedPublicFunctionCalls: PublicCallRequest[] = [];

  constructor(
    protected readonly contractAddress: AztecAddress,
    private readonly argsHash: Fr,
    private readonly txContext: TxContext,
    private readonly callContext: CallContext,
    /** Header of a block whose state is used during private execution (not the block the transaction is included in). */
    protected readonly historicalHeader: Header,
    /** List of transient auth witnesses to be used during this simulation */
    protected readonly authWitnesses: AuthWitness[],
    private readonly packedArgsCache: PackedArgsCache,
    private readonly noteCache: ExecutionNoteCache,
    protected readonly db: DBOracle,
    private readonly curve: Grumpkin,
    private node: AztecNode,
    protected sideEffectCounter: number = 0,
    protected log = createDebugLogger('aztec:simulator:client_execution_context'),
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

    const args = this.packedArgsCache.unpack(this.argsHash);

    if (args.length !== argumentsSize) {
      throw new Error('Invalid arguments size');
    }

    const fields = [
      ...this.callContext.toFields(),
      ...this.historicalHeader.toFields(),

      this.txContext.chainId,
      this.txContext.version,

      new Fr(this.sideEffectCounter),

      ...args,
    ];

    return toACVMWitness(0, fields);
  }

  /**
   * This function will populate readRequestPartialWitnesses which
   * here is just used to flag reads as "transient" for new notes created during this execution
   * or to flag non-transient reads with their leafIndex.
   * The KernelProver will use this to fully populate witnesses and provide hints to
   * the kernel regarding which commitments each transient read request corresponds to.
   * @param noteHashReadRequests - SideEffect containing Note hashed of the notes being read and counter.
   * @returns An array of partially filled in read request membership witnesses.
   */
  public getNoteHashReadRequestPartialWitnesses(noteHashReadRequests: SideEffect[]) {
    return noteHashReadRequests
      .filter(r => !r.isEmpty())
      .map(r => {
        const index = this.gotNotes.get(r.value.toBigInt());
        return index !== undefined
          ? NoteHashReadRequestMembershipWitness.empty(index)
          : NoteHashReadRequestMembershipWitness.emptyTransient();
      });
  }

  /**
   * Get the data for the newly created notes.
   * @param innerNoteHashes - Inner note hashes for the notes.
   */
  public getNewNotes(): NoteAndSlot[] {
    return this.newNotes;
  }

  /**
   * Return the encrypted logs emitted during this execution.
   */
  public getEncryptedLogs() {
    return new EncryptedFunctionL2Logs(this.encryptedLogs);
  }

  /**
   * Return the encrypted logs emitted during this execution.
   */
  public getUnencryptedLogs() {
    return new UnencryptedFunctionL2Logs(this.unencryptedLogs);
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
   * Pack the given arguments.
   * @param args - Arguments to pack
   */
  public packArguments(args: Fr[]): Promise<Fr> {
    return Promise.resolve(this.packedArgsCache.pack(args));
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
  public async getNotes(
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

    this.log(
      `Returning ${notes.length} notes for ${this.callContext.storageContractAddress} at ${storageSlot}: ${notes
        .map(n => `${n.nonce.toString()}:[${n.note.items.map(i => i.toString()).join(',')}]`)
        .join(', ')}`,
    );

    notes.forEach(n => {
      if (n.index !== undefined) {
        const siloedNoteHash = siloNoteHash(n.contractAddress, n.innerNoteHash);
        const uniqueSiloedNoteHash = computeUniqueCommitment(n.nonce, siloedNoteHash);
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
        // Should always be uniqueSiloedNoteHash when publicly created notes include nonces.
        const noteHashForReadRequest = n.nonce.isZero() ? siloedNoteHash : uniqueSiloedNoteHash;
        this.gotNotes.set(noteHashForReadRequest.value, n.index);
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
  public notifyCreatedNote(storageSlot: Fr, noteTypeId: Fr, noteItems: Fr[], innerNoteHash: Fr) {
    const note = new Note(noteItems);
    this.noteCache.addNewNote({
      contractAddress: this.callContext.storageContractAddress,
      storageSlot,
      nonce: Fr.ZERO, // Nonce cannot be known during private execution.
      note,
      siloedNullifier: undefined, // Siloed nullifier cannot be known for newly created note.
      innerNoteHash,
    });
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
  public notifyNullifiedNote(innerNullifier: Fr, innerNoteHash: Fr) {
    this.noteCache.nullifyNote(this.callContext.storageContractAddress, innerNullifier, innerNoteHash);
    return Promise.resolve();
  }

  /**
   * Encrypt a note and emit it as a log.
   * @param contractAddress - The contract address of the note.
   * @param storageSlot - The storage slot the note is at.
   * @param noteTypeId - The type ID of the note.
   * @param publicKey - The public key of the account that can decrypt the log.
   * @param log - The log contents.
   */
  public emitEncryptedLog(contractAddress: AztecAddress, storageSlot: Fr, noteTypeId: Fr, publicKey: Point, log: Fr[]) {
    const note = new Note(log);
    const l1NotePayload = new L1NotePayload(note, contractAddress, storageSlot, noteTypeId);
    const taggedNote = new TaggedNote(l1NotePayload);
    const encryptedNote = taggedNote.toEncryptedBuffer(publicKey, this.curve);
    this.encryptedLogs.push(new EncryptedL2Log(encryptedNote));
  }

  /**
   * Emit an unencrypted log.
   * @param log - The unencrypted log to be emitted.
   */
  public emitUnencryptedLog(log: UnencryptedL2Log) {
    this.unencryptedLogs.push(log);
    const text = log.toHumanReadable();
    this.log(`Emitted unencrypted log: "${text.length > 100 ? text.slice(0, 100) + '...' : text}"`);
  }

  #checkValidStaticCall(childExecutionResult: ExecutionResult) {
    if (
      childExecutionResult.callStackItem.publicInputs.newNoteHashes.some(item => !item.isEmpty()) ||
      childExecutionResult.callStackItem.publicInputs.newNullifiers.some(item => !item.isEmpty()) ||
      childExecutionResult.callStackItem.publicInputs.newL2ToL1Msgs.some(item => !item.isEmpty()) ||
      !childExecutionResult.callStackItem.publicInputs.encryptedLogPreimagesLength.equals(new Fr(4)) ||
      !childExecutionResult.callStackItem.publicInputs.unencryptedLogPreimagesLength.equals(new Fr(4))
    ) {
      throw new Error(`Static call cannot create new notes, emit L2->L1 messages or generate logs`);
    }
  }

  /**
   * Calls a private function as a nested execution.
   * @param targetContractAddress - The address of the contract to call.
   * @param functionSelector - The function selector of the function to call.
   * @param argsHash - The packed arguments to pass to the function.
   * @param sideEffectCounter - The side effect counter at the start of the call.
   * @param isStaticCall - Whether the call is a static call.
   * @param isStaticCall - Whether the call is a delegate call.
   * @returns The execution result.
   */
  async callPrivateFunction(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    this.log(
      `Calling private function ${this.contractAddress}:${functionSelector} from ${this.callContext.storageContractAddress}`,
    );

    isStaticCall = isStaticCall || this.callContext.isStaticCall;

    const targetArtifact = await this.db.getFunctionArtifact(targetContractAddress, functionSelector);
    const targetFunctionData = FunctionData.fromAbi(targetArtifact);

    const derivedTxContext = new TxContext(false, false, this.txContext.chainId, this.txContext.version);

    const derivedCallContext = await this.deriveCallContext(
      targetContractAddress,
      targetArtifact,
      sideEffectCounter,
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
      this.packedArgsCache,
      this.noteCache,
      this.db,
      this.curve,
      this.node,
      sideEffectCounter,
    );

    const childExecutionResult = await executePrivateFunction(
      context,
      targetArtifact,
      targetContractAddress,
      targetFunctionData,
    );

    if (isStaticCall) {
      this.#checkValidStaticCall(childExecutionResult);
    }

    this.nestedExecutions.push(childExecutionResult);

    return childExecutionResult.callStackItem;
  }

  /**
   * Creates a PublicCallStackItem object representing the request to call a public function. No function
   * is actually called, since that must happen on the sequencer side. All the fields related to the result
   * of the execution are empty.
   * @param targetContractAddress - The address of the contract to call.
   * @param functionSelector - The function selector of the function to call.
   * @param argsHash - The packed arguments to pass to the function.
   * @param sideEffectCounter - The side effect counter at the start of the call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The public call stack item with the request information.
   */
  public async enqueuePublicFunctionCall(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ): Promise<PublicCallRequest> {
    isStaticCall = isStaticCall || this.callContext.isStaticCall;

    const targetArtifact = await this.db.getFunctionArtifact(targetContractAddress, functionSelector);
    const derivedCallContext = await this.deriveCallContext(
      targetContractAddress,
      targetArtifact,
      sideEffectCounter,
      isDelegateCall,
      isStaticCall,
    );
    const args = this.packedArgsCache.unpack(argsHash);
    const enqueuedRequest = PublicCallRequest.from({
      args,
      callContext: derivedCallContext,
      parentCallContext: this.callContext,
      functionData: FunctionData.fromAbi(targetArtifact),
      contractAddress: targetContractAddress,
    });

    // TODO($846): if enqueued public calls are associated with global
    // side-effect counter, that will leak info about how many other private
    // side-effects occurred in the TX. Ultimately the private kernel should
    // just output everything in the proper order without any counters.
    this.log(
      `Enqueued call to public function (with side-effect counter #${sideEffectCounter}) ${targetContractAddress}:${functionSelector}(${targetArtifact.name})`,
    );

    this.enqueuedPublicFunctionCalls.push(enqueuedRequest);

    return enqueuedRequest;
  }

  /**
   * Derives the call context for a nested execution.
   * @param targetContractAddress - The address of the contract being called.
   * @param targetArtifact - The artifact of the function being called.
   * @param startSideEffectCounter - The side effect counter at the start of the call.
   * @param isDelegateCall - Whether the call is a delegate call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The derived call context.
   */
  private async deriveCallContext(
    targetContractAddress: AztecAddress,
    targetArtifact: FunctionArtifact,
    startSideEffectCounter: number,
    isDelegateCall = false,
    isStaticCall = false,
  ) {
    const portalContractAddress = await this.db.getPortalContractAddress(targetContractAddress);
    return new CallContext(
      isDelegateCall ? this.callContext.msgSender : this.contractAddress,
      isDelegateCall ? this.contractAddress : targetContractAddress,
      portalContractAddress,
      FunctionSelector.fromNameAndParameters(targetArtifact.name, targetArtifact.parameters),
      isDelegateCall,
      isStaticCall,
      startSideEffectCounter,
    );
  }

  /**
   * Read the public storage data.
   * @param startStorageSlot - The starting storage slot.
   * @param numberOfElements - Number of elements to read from the starting storage slot.
   */
  public async storageRead(startStorageSlot: Fr, numberOfElements: number): Promise<Fr[]> {
    // TODO(#4320): This is a hack to work around not having directly access to the public data tree but
    // still having access to the witnesses
    const bn = await this.db.getBlockNumber();

    const values = [];
    for (let i = 0n; i < numberOfElements; i++) {
      const storageSlot = new Fr(startStorageSlot.value + i);
      const leafSlot = computePublicDataTreeLeafSlot(this.callContext.storageContractAddress, storageSlot);
      const witness = await this.db.getPublicDataTreeWitness(bn, leafSlot);
      if (!witness) {
        throw new Error(`No witness for slot ${storageSlot.toString()}`);
      }
      const value = witness.leafPreimage.value;
      this.log(`Oracle storage read: slot=${storageSlot.toString()} value=${value}`);
      values.push(value);
    }
    return values;
  }
}
