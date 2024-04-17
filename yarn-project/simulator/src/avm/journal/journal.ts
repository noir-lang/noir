import { UnencryptedL2Log } from '@aztec/circuit-types';
import { AztecAddress, EthAddress, L2ToL1Message } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { type HostStorage } from './host_storage.js';
import { Nullifiers } from './nullifiers.js';
import { PublicStorage } from './public_storage.js';
import { WorldStateAccessTrace } from './trace.js';
import {
  type TracedL1toL2MessageCheck,
  type TracedNoteHash,
  type TracedNoteHashCheck,
  type TracedNullifier,
  type TracedNullifierCheck,
  type TracedPublicStorageRead,
  type TracedPublicStorageWrite,
  type TracedUnencryptedL2Log,
} from './trace_types.js';

/**
 * Data held within the journal
 */
export type JournalData = {
  storageWrites: TracedPublicStorageWrite[];
  storageReads: TracedPublicStorageRead[];

  noteHashChecks: TracedNoteHashCheck[];
  newNoteHashes: TracedNoteHash[];
  nullifierChecks: TracedNullifierCheck[];
  newNullifiers: TracedNullifier[];
  l1ToL2MessageChecks: TracedL1toL2MessageCheck[];

  newL1Messages: L2ToL1Message[];
  newLogs: UnencryptedL2Log[];
  newLogsHashes: TracedUnencryptedL2Log[];
  /** contract address -\> key -\> value */
  currentStorageValue: Map<bigint, Map<bigint, Fr>>;
};

/**
 * A class to manage persistable AVM state for contract calls.
 * Maintains a cache of the current world state,
 * a trace of all world state accesses, and a list of accrued substate items.
 *
 * The simulator should make any world state and accrued substate queries through this object.
 *
 * Manages merging of successful/reverted child state into current state.
 */
export class AvmPersistableStateManager {
  private readonly log: DebugLogger = createDebugLogger('aztec:avm_simulator:state_manager');
  /** Reference to node storage */
  public readonly hostStorage: HostStorage;

  // TODO: make members private once this is not used in transitional_adaptors.ts.
  /** World State */
  /** Public storage, including cached writes */
  public publicStorage: PublicStorage;
  /** Nullifier set, including cached/recently-emitted nullifiers */
  public nullifiers: Nullifiers;

  /** World State Access Trace */
  public trace: WorldStateAccessTrace;

  /** Accrued Substate **/
  public newL1Messages: L2ToL1Message[] = [];
  public newLogs: UnencryptedL2Log[] = [];

  constructor(hostStorage: HostStorage, parent?: AvmPersistableStateManager) {
    this.hostStorage = hostStorage;
    this.publicStorage = new PublicStorage(hostStorage.publicStateDb, parent?.publicStorage);
    this.nullifiers = new Nullifiers(hostStorage.commitmentsDb, parent?.nullifiers);
    this.trace = new WorldStateAccessTrace(parent?.trace);
  }

  /**
   * Create a new state manager forked from this one
   */
  public fork() {
    return new AvmPersistableStateManager(this.hostStorage, this);
  }

  /**
   * Write to public storage, journal/trace the write.
   *
   * @param storageAddress - the address of the contract whose storage is being written to
   * @param slot - the slot in the contract's storage being written to
   * @param value - the value being written to the slot
   */
  public writeStorage(storageAddress: Fr, slot: Fr, value: Fr) {
    this.log.debug(`storage(${storageAddress})@${slot} <- ${value}`);
    // Cache storage writes for later reference/reads
    this.publicStorage.write(storageAddress, slot, value);
    // Trace all storage writes (even reverted ones)
    this.trace.tracePublicStorageWrite(storageAddress, slot, value);
  }

  /**
   * Read from public storage, trace the read.
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param slot - the slot in the contract's storage being read from
   * @returns the latest value written to slot, or 0 if never written to before
   */
  public async readStorage(storageAddress: Fr, slot: Fr): Promise<Fr> {
    const [exists, value] = await this.publicStorage.read(storageAddress, slot);
    this.log.debug(`storage(${storageAddress})@${slot} ?? value: ${value}, exists: ${exists}.`);
    // We want to keep track of all performed reads (even reverted ones)
    this.trace.tracePublicStorageRead(storageAddress, slot, value, exists);
    return Promise.resolve(value);
  }

  // TODO(4886): We currently don't silo note hashes.
  /**
   * Check if a note hash exists at the given leaf index, trace the check.
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param noteHash - the unsiloed note hash being checked
   * @param leafIndex - the leaf index being checked
   * @returns true if the note hash exists at the given leaf index, false otherwise
   */
  public async checkNoteHashExists(storageAddress: Fr, noteHash: Fr, leafIndex: Fr): Promise<boolean> {
    const gotLeafIndex = await this.hostStorage.commitmentsDb.getCommitmentIndex(noteHash);
    const exists = gotLeafIndex === leafIndex.toBigInt();
    this.log.debug(`noteHashes(${storageAddress})@${noteHash} ?? leafIndex: ${leafIndex}, exists: ${exists}.`);
    this.trace.traceNoteHashCheck(storageAddress, noteHash, exists, leafIndex);
    return Promise.resolve(exists);
  }

  /**
   * Write a note hash, trace the write.
   * @param noteHash - the unsiloed note hash to write
   */
  public writeNoteHash(storageAddress: Fr, noteHash: Fr) {
    this.log.debug(`noteHashes(${storageAddress}) += @${noteHash}.`);
    this.trace.traceNewNoteHash(storageAddress, noteHash);
  }

  /**
   * Check if a nullifier exists, trace the check.
   * @param storageAddress - address of the contract that the nullifier is associated with
   * @param nullifier - the unsiloed nullifier to check
   * @returns exists - whether the nullifier exists in the nullifier set
   */
  public async checkNullifierExists(storageAddress: Fr, nullifier: Fr): Promise<boolean> {
    const [exists, isPending, leafIndex] = await this.nullifiers.checkExists(storageAddress, nullifier);
    this.log.debug(
      `nullifiers(${storageAddress})@${nullifier} ?? leafIndex: ${leafIndex}, pending: ${isPending}, exists: ${exists}.`,
    );
    this.trace.traceNullifierCheck(storageAddress, nullifier, exists, isPending, leafIndex);
    return Promise.resolve(exists);
  }

  /**
   * Write a nullifier to the nullifier set, trace the write.
   * @param storageAddress - address of the contract that the nullifier is associated with
   * @param nullifier - the unsiloed nullifier to write
   */
  public async writeNullifier(storageAddress: Fr, nullifier: Fr) {
    this.log.debug(`nullifiers(${storageAddress}) += ${nullifier}.`);
    // Cache pending nullifiers for later access
    await this.nullifiers.append(storageAddress, nullifier);
    // Trace all nullifier creations (even reverted ones)
    this.trace.traceNewNullifier(storageAddress, nullifier);
  }

  /**
   * Check if an L1 to L2 message exists, trace the check.
   * @param msgHash - the message hash to check existence of
   * @param msgLeafIndex - the message leaf index to use in the check
   * @returns exists - whether the message exists in the L1 to L2 Messages tree
   */
  public async checkL1ToL2MessageExists(msgHash: Fr, msgLeafIndex: Fr): Promise<boolean> {
    let exists = false;
    try {
      // The following 2 values are used to compute a message nullifier. Given that here we do not care about getting
      // non-nullified messages we can just pass in random values and the nullifier check will effectively be ignored
      // (no nullifier will be found).
      const ignoredContractAddress = AztecAddress.random();
      const ignoredSecret = Fr.random();
      const gotMessage = await this.hostStorage.commitmentsDb.getL1ToL2MembershipWitness(
        ignoredContractAddress,
        msgHash,
        ignoredSecret,
      );
      exists = gotMessage !== undefined && gotMessage.index == msgLeafIndex.toBigInt();
    } catch {
      // error getting message - doesn't exist!
      exists = false;
    }
    this.log.debug(`l1ToL2Messages(${msgHash})@${msgLeafIndex} ?? exists: ${exists}.`);
    this.trace.traceL1ToL2MessageCheck(msgHash, msgLeafIndex, exists);
    return Promise.resolve(exists);
  }

  /**
   * Write an L2 to L1 message.
   * @param recipient - L1 contract address to send the message to.
   * @param content - Message content.
   */
  public writeL1Message(recipient: EthAddress | Fr, content: Fr) {
    this.log.debug(`L1Messages(${recipient}) += ${content}.`);
    const recipientAddress = recipient instanceof EthAddress ? recipient : EthAddress.fromField(recipient);
    this.newL1Messages.push(new L2ToL1Message(recipientAddress, content));
  }

  public writeLog(contractAddress: Fr, event: Fr, log: Fr[]) {
    this.log.debug(`UnencryptedL2Log(${contractAddress}) += event ${event} with ${log.length} fields.`);
    const L2log = new UnencryptedL2Log(
      AztecAddress.fromField(contractAddress),
      EventSelector.fromField(event),
      Buffer.concat(log.map(f => f.toBuffer())),
    );
    this.newLogs.push(L2log);
    this.trace.traceNewLog(Fr.fromBuffer(L2log.hash()));
  }

  /**
   * Accept nested world state modifications, merging in its trace and accrued substate
   */
  public acceptNestedCallState(nestedJournal: AvmPersistableStateManager) {
    // Merge Public Storage
    this.publicStorage.acceptAndMerge(nestedJournal.publicStorage);

    // Merge World State Access Trace
    this.trace.acceptAndMerge(nestedJournal.trace);

    // Accrued Substate
    this.newL1Messages = this.newL1Messages.concat(nestedJournal.newL1Messages);
    this.newLogs = this.newLogs.concat(nestedJournal.newLogs);
  }

  /**
   * Reject nested world state, merging in its trace, but not accepting any state modifications
   */
  public rejectNestedCallState(nestedJournal: AvmPersistableStateManager) {
    // Merge World State Access Trace
    this.trace.acceptAndMerge(nestedJournal.trace);
  }

  /**
   * Access the current state of the journal
   *
   * @returns a JournalData object
   */
  public flush(): JournalData {
    return {
      noteHashChecks: this.trace.noteHashChecks,
      newNoteHashes: this.trace.newNoteHashes,
      nullifierChecks: this.trace.nullifierChecks,
      newNullifiers: this.trace.newNullifiers,
      l1ToL2MessageChecks: this.trace.l1ToL2MessageChecks,
      newL1Messages: this.newL1Messages,
      newLogs: this.newLogs,
      newLogsHashes: this.trace.newLogsHashes,
      currentStorageValue: this.publicStorage.getCache().cachePerContract,
      storageReads: this.trace.publicStorageReads,
      storageWrites: this.trace.publicStorageWrites,
    };
  }
}
