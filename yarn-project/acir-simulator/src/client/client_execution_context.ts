import { CircuitsWasm, HistoricBlockData, ReadRequestMembershipWitness, TxContext } from '@aztec/circuits.js';
import { computeUniqueCommitment, siloCommitment } from '@aztec/circuits.js/abis';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import {
  ACVMField,
  ONE_ACVM_FIELD,
  ZERO_ACVM_FIELD,
  fromACVMField,
  toACVMField,
  toAcvmL1ToL2MessageLoadOracleInputs,
} from '../acvm/index.js';
import { PackedArgsCache } from '../common/packed_args_cache.js';
import { DBOracle } from './db_oracle.js';
import { ExecutionNoteCache } from './execution_note_cache.js';
import { NewNoteData } from './execution_result.js';
import { pickNotes } from './pick_notes.js';

/**
 * The execution context for a client tx simulation.
 */
export class ClientTxExecutionContext {
  /**
   * New notes created during this execution.
   * It's possible that a note in this list has been nullified (in the same or other executions) and doen't exist in the ExecutionNoteCache and the final proof data.
   * But we still include those notes in the execution result because their commitments are still in the public inputs of this execution.
   * This information is only for references (currently used for tests), and is not used for any sort of constrains.
   * Users can also use this to get a clearer idea of what's happened during a simulation.
   */
  private newNotes: NewNoteData[] = [];
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

  /** Logger instance */
  private logger = createDebugLogger('aztec:simulator:execution_context');

  constructor(
    /**  The database oracle. */
    public db: DBOracle,
    /** The tx context. */
    public txContext: TxContext,
    /** Data required to reconstruct the block hash, it contains historic roots. */
    public historicBlockData: HistoricBlockData,
    /** The cache of packed arguments */
    public packedArgsCache: PackedArgsCache,
    private noteCache: ExecutionNoteCache,
    private log = createDebugLogger('aztec:simulator:client_execution_context'),
  ) {}

  /**
   * Create context for nested executions.
   * @returns ClientTxExecutionContext
   */
  public extend() {
    return new ClientTxExecutionContext(
      this.db,
      this.txContext,
      this.historicBlockData,
      this.packedArgsCache,
      this.noteCache,
    );
  }

  /**
   * This function will populate readRequestPartialWitnesses which
   * here is just used to flag reads as "transient" for new notes created during this execution
   * or to flag non-transient reads with their leafIndex.
   * The KernelProver will use this to fully populate witnesses and provide hints to
   * the kernel regarding which commitments each transient read request corresponds to.
   * @param readRequests - Note hashed of the notes being read.
   * @returns An array of partially filled in read request membership witnesses.
   */
  public getReadRequestPartialWitnesses(readRequests: Fr[]) {
    return readRequests
      .filter(r => !r.isZero())
      .map(r => {
        const index = this.gotNotes.get(r.value);
        return index !== undefined
          ? ReadRequestMembershipWitness.empty(index)
          : ReadRequestMembershipWitness.emptyTransient();
      });
  }

  /**
   * Get the data for the newly created notes.
   * @param innerNoteHashes - Inner note hashes for the notes.
   */
  public getNewNotes(): NewNoteData[] {
    return this.newNotes;
  }

  /**
   * For getting secret key.
   * @param contractAddress - The contract address.
   * @param ownerX - The x coordinate of the owner's public key.
   * @param ownerY - The y coordinate of the owner's public key.
   * @returns The secret key of the owner as a pair of ACVM fields.
   */
  public async getSecretKey(contractAddress: AztecAddress, ownerX: ACVMField, ownerY: ACVMField) {
    const secretKey = await this.db.getSecretKey(
      contractAddress,
      new Point(fromACVMField(ownerX), fromACVMField(ownerY)),
    );
    return [toACVMField(secretKey.high), toACVMField(secretKey.low)];
  }

  /**
   * Gets some notes for a contract address and storage slot.
   * Returns a flattened array containing real-note-count and note preimages.
   *
   * @remarks
   *
   * Check for pending notes with matching address/slot.
   * Real notes coming from DB will have a leafIndex which
   * represents their index in the private data tree.
   *
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param numSelects - The number of valid selects in selectBy and selectValues.
   * @param selectBy - An array of indices of the fields to selects.
   * @param selectValues - The values to match.
   * @param sortBy - An array of indices of the fields to sort.
   * @param sortOrder - The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * @param limit - The number of notes to retrieve per query.
   * @param offset - The starting index for pagination.
   * @param returnSize - The return size.
   * @returns Flattened array of ACVMFields (format expected by Noir/ACVM) containing:
   * count - number of real (non-padding) notes retrieved,
   * contractAddress - the contract address,
   * preimages - the real note preimages retrieved, and
   * paddedZeros - zeros to ensure an array with length returnSize expected by Noir circuit
   */
  public async getNotes(
    contractAddress: AztecAddress,
    storageSlot: ACVMField,
    numSelects: number,
    selectBy: ACVMField[],
    selectValues: ACVMField[],
    sortBy: ACVMField[],
    sortOrder: ACVMField[],
    limit: number,
    offset: number,
    returnSize: number,
  ): Promise<ACVMField[]> {
    const storageSlotField = fromACVMField(storageSlot);

    // Nullified pending notes are already removed from the list.
    const pendingNotes = this.noteCache.getNotes(contractAddress, storageSlotField);

    const pendingNullifiers = this.noteCache.getNullifiers(contractAddress);
    const dbNotes = await this.db.getNotes(contractAddress, storageSlotField);
    const dbNotesFiltered = dbNotes.filter(n => !pendingNullifiers.has((n.siloedNullifier as Fr).value));

    const notes = pickNotes([...dbNotesFiltered, ...pendingNotes], {
      selects: selectBy
        .slice(0, numSelects)
        .map((fieldIndex, i) => ({ index: +fieldIndex, value: fromACVMField(selectValues[i]) })),
      sorts: sortBy.map((fieldIndex, i) => ({ index: +fieldIndex, order: +sortOrder[i] })),
      limit,
      offset,
    });

    this.logger(
      `Returning ${notes.length} notes for ${contractAddress} at ${storageSlotField}: ${notes
        .map(n => `${n.nonce.toString()}:[${n.preimage.map(i => i.toString()).join(',')}]`)
        .join(', ')}`,
    );

    const wasm = await CircuitsWasm.get();
    notes.forEach(n => {
      if (n.index !== undefined) {
        const siloedNoteHash = siloCommitment(wasm, n.contractAddress, n.innerNoteHash);
        const uniqueSiloedNoteHash = computeUniqueCommitment(wasm, n.nonce, siloedNoteHash);
        this.gotNotes.set(uniqueSiloedNoteHash.value, n.index);
      }
    });

    // TODO: notice, that if we don't have a note in our DB, we don't know how big the preimage needs to be, and so we don't actually know how many dummy notes to return, or big to make those dummy notes, or where to position `is_some` booleans to inform the noir program that _all_ the notes should be dummies.
    // By a happy coincidence, a `0` field is interpreted as `is_none`, and since in this case (of an empty db) we'll return all zeros (paddedZeros), the noir program will treat the returned data as all dummies, but this is luck. Perhaps a preimage size should be conveyed by the get_notes noir oracle?
    const preimageLength = notes?.[0]?.preimage.length ?? 0;
    if (
      !notes.every(({ preimage }) => {
        return preimageLength === preimage.length;
      })
    )
      throw new Error('Preimages for a particular note type should all be the same length');

    // Combine pending and db preimages into a single flattened array.
    const isSome = new Fr(1); // Boolean. Indicates whether the Noir Option<Note>::is_some();

    const realNotePreimages = notes.flatMap(({ nonce, preimage }) => [nonce, isSome, ...preimage]);

    const returnHeaderLength = 2; // is for the header values: `notes.length` and `contractAddress`.
    const extraPreimageLength = 2; // is for the nonce and isSome fields.
    const extendedPreimageLength = preimageLength + extraPreimageLength;
    const numRealNotes = notes.length;
    const numReturnNotes = Math.floor((returnSize - returnHeaderLength) / extendedPreimageLength);
    const numDummyNotes = numReturnNotes - numRealNotes;

    const dummyNotePreimage = Array(extendedPreimageLength).fill(Fr.ZERO);
    const dummyNotePreimages = Array(numDummyNotes)
      .fill(dummyNotePreimage)
      .flatMap(note => note);

    const paddedZeros = Array(
      Math.max(0, returnSize - returnHeaderLength - realNotePreimages.length - dummyNotePreimages.length),
    ).fill(Fr.ZERO);

    return [notes.length, contractAddress, ...realNotePreimages, ...dummyNotePreimages, ...paddedZeros].map(v =>
      toACVMField(v),
    );
  }

  /**
   * Keep track of the new note created during execution.
   * It can be used in subsequent calls (or transactions when chaining txs is possible).
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param preimage - The preimage of the new note.
   * @param innerNoteHash - The inner note hash of the new note.
   * @returns
   */
  public handleNewNote(
    contractAddress: AztecAddress,
    storageSlot: ACVMField,
    preimage: ACVMField[],
    innerNoteHash: ACVMField,
  ) {
    this.noteCache.addNewNote({
      contractAddress,
      storageSlot: fromACVMField(storageSlot),
      nonce: Fr.ZERO, // Nonce cannot be known during private execution.
      preimage: preimage.map(f => fromACVMField(f)),
      siloedNullifier: undefined, // Siloed nullifier cannot be known for newly created note.
      innerNoteHash: fromACVMField(innerNoteHash),
    });
    this.newNotes.push({
      storageSlot: fromACVMField(storageSlot),
      preimage: preimage.map(f => fromACVMField(f)),
    });
  }

  /**
   * Adding a siloed nullifier into the current set of all pending nullifiers created
   * within the current transaction/execution.
   * @param contractAddress - The contract address.
   * @param innerNullifier - The pending nullifier to add in the list (not yet siloed by contract address).
   * @param innerNoteHash - The inner note hash of the new note.
   * @param contractAddress - The contract address
   */
  public async handleNullifiedNote(contractAddress: AztecAddress, innerNullifier: ACVMField, innerNoteHash: ACVMField) {
    await this.noteCache.nullifyNote(contractAddress, fromACVMField(innerNullifier), fromACVMField(innerNoteHash));
  }

  /**
   * Fetches the a message from the db, given its key.
   * @param msgKey - A buffer representing the message key.
   * @returns The l1 to l2 message data
   */
  public async getL1ToL2Message(msgKey: Fr): Promise<ACVMField[]> {
    const messageInputs = await this.db.getL1ToL2Message(msgKey);
    return toAcvmL1ToL2MessageLoadOracleInputs(messageInputs, this.historicBlockData.l1ToL2MessagesTreeRoot);
  }

  /**
   * Fetches a path to prove existence of a commitment in the db, given its contract side commitment (before silo).
   * @param contractAddress - The contract address.
   * @param nonce - The nonce of the note.
   * @param innerNoteHash - The inner note hash of the note.
   * @returns 1 if (persistent or transient) note hash exists, 0 otherwise. Value is in ACVMField form.
   */
  public async checkNoteHashExists(
    contractAddress: AztecAddress,
    nonce: ACVMField,
    innerNoteHash: ACVMField,
  ): Promise<ACVMField> {
    const nonceField = fromACVMField(nonce);
    const innerNoteHashField = fromACVMField(innerNoteHash);
    if (nonceField.isZero()) {
      // If nonce is 0, we are looking for a new note created in this transaction.
      const exists = this.noteCache.checkNoteExists(contractAddress, innerNoteHashField);
      if (exists) {
        return ONE_ACVM_FIELD;
      }
      // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
      // Currently it can also be a note created from public if nonce is 0.
      // If we can't find a matching new note, keep looking for the match from the notes created in previous transactions.
    }

    // If nonce is zero, SHOULD only be able to reach this point if note was publicly created
    const wasm = await CircuitsWasm.get();
    let noteHashToLookUp = siloCommitment(wasm, contractAddress, innerNoteHashField);
    if (!nonceField.isZero()) {
      noteHashToLookUp = computeUniqueCommitment(wasm, nonceField, noteHashToLookUp);
    }

    const index = await this.db.getCommitmentIndex(noteHashToLookUp);
    if (index !== undefined) {
      this.gotNotes.set(noteHashToLookUp.value, index);
    }
    return index !== undefined ? ONE_ACVM_FIELD : ZERO_ACVM_FIELD;
  }
}
