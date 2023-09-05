import { CircuitsWasm, HistoricBlockData, ReadRequestMembershipWitness, TxContext } from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/abis';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import {
  ACVMField,
  fromACVMField,
  toACVMField,
  toAcvmCommitmentLoadOracleInputs,
  toAcvmL1ToL2MessageLoadOracleInputs,
} from '../acvm/index.js';
import { PackedArgsCache } from '../common/packed_args_cache.js';
import { DBOracle, PendingNoteData } from './db_oracle.js';
import { pickNotes } from './pick_notes.js';

/**
 * The execution context for a client tx simulation.
 */
export class ClientTxExecutionContext {
  // Note: not forwarded to nested contexts via `extend()` because these witnesses
  // are meant to be maintained on a per-call basis as they should mirror read requests
  // output by an app circuit via public inputs.
  private readRequestPartialWitnesses: ReadRequestMembershipWitness[] = [];

  /** Logger instance */
  private logger = createDebugLogger('aztec:simulator:execution_context');

  constructor(
    /**  The database oracle. */
    public db: DBOracle,
    /** The tx nullifier, which is also the first nullifier. This will be used to compute the nonces for pending notes. */
    private txNullifier: Fr,
    /** The tx context. */
    public txContext: TxContext,
    /** Data required to reconstruct the block hash, it contains historic roots. */
    public historicBlockData: HistoricBlockData,
    /** The cache of packed arguments */
    public packedArgsCache: PackedArgsCache,
    /** Pending notes created (and not nullified) up to current point in execution.
     *  If a nullifier for a note in this list is emitted, the note will be REMOVED. */
    private pendingNotes: PendingNoteData[] = [],
    /** The list of nullifiers created in this transaction. The commitment/note which is nullified
     *  might be pending or not (i.e., was generated in a previous transaction)
     *  Note that their value (bigint representation) is used because Frs cannot be looked up in Sets. */
    private pendingNullifiers: Set<bigint> = new Set<bigint>(),

    private log = createDebugLogger('aztec:simulator:client_execution_context'),
  ) {}

  /**
   * Create context for nested executions.
   * @returns ClientTxExecutionContext
   */
  public extend() {
    return new ClientTxExecutionContext(
      this.db,
      this.txNullifier,
      this.txContext,
      this.historicBlockData,
      this.packedArgsCache,
      this.pendingNotes,
      this.pendingNullifiers,
    );
  }

  /**
   * For getting accumulated data.
   * @returns An array of partially filled in read request membership witnesses.
   */
  public getReadRequestPartialWitnesses() {
    return this.readRequestPartialWitnesses;
  }

  /**
   * For getting secret key.
   * @param contractAddress - The contract address.
   * @param ownerX - The x coordinate of the owner's public key.
   * @param ownerY - The y coordinate of the owner's public key.
   * @returns The secret key of the owner.
   */
  public async getSecretKey(contractAddress: AztecAddress, ownerX: ACVMField, ownerY: ACVMField) {
    return toACVMField(
      (await this.db.getSecretKey(contractAddress, new Point(fromACVMField(ownerX), fromACVMField(ownerY)))).value,
    );
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
   * This function will populate this.readRequestPartialWitnesses which
   * here is just used to flag reads as "transient" (one in getPendingNotes)
   * or to flag non-transient reads with their leafIndex.
   * The KernelProver will use this to fully populate witnesses and provide hints to
   * the kernel regarding which commitments each transient read request corresponds to.
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

    const pendingNotes = this.pendingNotes.filter(
      n => n.contractAddress.equals(contractAddress) && n.storageSlot.equals(storageSlotField),
    );

    const dbNotes = await this.db.getNotes(contractAddress, storageSlotField);

    const dbNotesFiltered = dbNotes.filter(n => !this.pendingNullifiers.has((n.siloedNullifier as Fr).value));

    // Nullified pending notes are already removed from the list.
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

    // Add a partial witness for each note.
    // It contains the note index for db notes. And flagged as transient for pending notes.
    notes.forEach(({ index }) => {
      this.readRequestPartialWitnesses.push(
        index !== undefined ? ReadRequestMembershipWitness.empty(index) : ReadRequestMembershipWitness.emptyTransient(),
      );
    });

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
   * @param commitment - The commitment.
   * @returns The commitment data.
   */
  public async getCommitment(contractAddress: AztecAddress, commitment: ACVMField) {
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): only works
    // for noteHashes/commitments created by public functions! Once public kernel or
    // base rollup circuit injects nonces, this can be used with commitments created by
    // private functions as well.
    const commitmentInputs = await this.db.getCommitmentOracle(contractAddress, fromACVMField(commitment));
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1029): support pending commitments here
    this.readRequestPartialWitnesses.push(ReadRequestMembershipWitness.empty(commitmentInputs.index));
    return toAcvmCommitmentLoadOracleInputs(commitmentInputs, this.historicBlockData.privateDataTreeRoot);
  }

  /**
   * Process new note created during execution.
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param preimage - new note preimage.
   * @param innerNoteHash - inner note hash
   */
  public pushNewNote(contractAddress: AztecAddress, storageSlot: Fr, preimage: Fr[], innerNoteHash: Fr) {
    this.pendingNotes.push({
      contractAddress,
      storageSlot: storageSlot,
      nonce: Fr.ZERO, // nonce is cannot be known during private execution
      preimage,
      innerNoteHash,
    });
  }

  /**
   * Adding a siloed nullifier into the current set of all pending nullifiers created
   * within the current transaction/execution.
   * @param innerNullifier - The pending nullifier to add in the list (not yet siloed by contract address).
   * @param contractAddress - The contract address
   */
  public async pushNewNullifier(innerNullifier: Fr, contractAddress: AztecAddress) {
    const wasm = await CircuitsWasm.get();
    const siloedNullifier = siloNullifier(wasm, contractAddress, innerNullifier);
    this.pendingNullifiers.add(siloedNullifier.value);
  }

  /**
   * Update the list of pending notes by chopping a note which is nullified.
   * The identifier used to determine matching is the inner note hash value.
   * However, we adopt a defensive approach and ensure that the contract address
   * and storage slot do match.
   * Note that this method might be called with an innerNoteHash referring to
   * a note created in a previous transaction which will result in this array
   * of pending notes left unchanged.
   * @param innerNoteHash - The inner note hash value to which note will be chopped.
   * @param contractAddress - The contract address
   * @param storageSlot - The storage slot as a Field Fr element
   */
  public nullifyPendingNotes(innerNoteHash: Fr, contractAddress: AztecAddress, storageSlot: Fr) {
    // IMPORTANT: We do need an in-place array mutation of this.pendingNotes as this array is shared
    // by reference among the nested calls. That is the main reason for 'splice' usage below.
    this.pendingNotes.splice(
      0,
      this.pendingNotes.length,
      ...this.pendingNotes.filter(
        n =>
          !(
            n.innerNoteHash.equals(innerNoteHash) &&
            n.contractAddress.equals(contractAddress) &&
            n.storageSlot.equals(storageSlot)
          ),
      ),
    );
  }
}
