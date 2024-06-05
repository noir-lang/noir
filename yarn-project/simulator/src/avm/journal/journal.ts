// TODO(5818): Rename file and all uses of "journal"
import { UnencryptedL2Log } from '@aztec/circuit-types';
import {
  AztecAddress,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  EthAddress,
  L2ToL1Message,
  LogHash,
  NoteHash,
  Nullifier,
  ReadRequest,
} from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import { type PublicExecutionResult } from '../../index.js';
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

// TODO:(5818): do we need this type anymore?
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

  sideEffectCounter: number;
};

// TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
export type PartialPublicExecutionResult = {
  noteHashReadRequests: ReadRequest[];
  nullifierReadRequests: ReadRequest[];
  nullifierNonExistentReadRequests: ReadRequest[];
  l1ToL2MsgReadRequests: ReadRequest[];
  newNoteHashes: NoteHash[];
  newL2ToL1Messages: L2ToL1Message[];
  startSideEffectCounter: number;
  newNullifiers: Nullifier[];
  contractStorageReads: ContractStorageRead[];
  contractStorageUpdateRequests: ContractStorageUpdateRequest[];
  unencryptedLogsHashes: LogHash[];
  unencryptedLogs: UnencryptedL2Log[];
  allUnencryptedLogs: UnencryptedL2Log[];
  nestedExecutions: PublicExecutionResult[];
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

  // TODO(5818): make members private once this is not used in transitional_adaptors.ts.
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

  // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
  public transitionalExecutionResult: PartialPublicExecutionResult;

  constructor(hostStorage: HostStorage, parent?: AvmPersistableStateManager) {
    this.hostStorage = hostStorage;
    this.publicStorage = new PublicStorage(hostStorage.publicStateDb, parent?.publicStorage);
    this.nullifiers = new Nullifiers(hostStorage.commitmentsDb, parent?.nullifiers);
    this.trace = new WorldStateAccessTrace(parent?.trace);

    this.transitionalExecutionResult = {
      noteHashReadRequests: [],
      nullifierReadRequests: [],
      nullifierNonExistentReadRequests: [],
      l1ToL2MsgReadRequests: [],
      newNoteHashes: [],
      newL2ToL1Messages: [],
      startSideEffectCounter: this.trace.accessCounter,
      newNullifiers: [],
      contractStorageReads: [],
      contractStorageUpdateRequests: [],
      unencryptedLogsHashes: [],
      unencryptedLogs: [],
      allUnencryptedLogs: [],
      nestedExecutions: [],
    };
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
    this.log.debug(`Storage write (address=${storageAddress}, slot=${slot}): value=${value}`);
    // Cache storage writes for later reference/reads
    this.publicStorage.write(storageAddress, slot, value);

    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    // The current info to the kernel clears any previous read or write request.
    this.transitionalExecutionResult.contractStorageReads =
      this.transitionalExecutionResult.contractStorageReads.filter(
        read => !read.storageSlot.equals(slot) || !read.contractAddress!.equals(storageAddress),
      );
    this.transitionalExecutionResult.contractStorageUpdateRequests =
      this.transitionalExecutionResult.contractStorageUpdateRequests.filter(
        update => !update.storageSlot.equals(slot) || !update.contractAddress!.equals(storageAddress),
      );
    this.transitionalExecutionResult.contractStorageUpdateRequests.push(
      new ContractStorageUpdateRequest(slot, value, this.trace.accessCounter, storageAddress),
    );

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
    const { value, exists, cached } = await this.publicStorage.read(storageAddress, slot);
    this.log.debug(
      `Storage read  (address=${storageAddress}, slot=${slot}): value=${value}, exists=${exists}, cached=${cached}`,
    );

    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    // The current info to the kernel kernel does not consider cached reads.
    if (!cached) {
      // The current info to the kernel removes any previous reads to the same slot.
      this.transitionalExecutionResult.contractStorageReads =
        this.transitionalExecutionResult.contractStorageReads.filter(
          read => !read.storageSlot.equals(slot) || !read.contractAddress!.equals(storageAddress),
        );
      this.transitionalExecutionResult.contractStorageReads.push(
        new ContractStorageRead(slot, value, this.trace.accessCounter, storageAddress),
      );
    }

    // We want to keep track of all performed reads (even reverted ones)
    this.trace.tracePublicStorageRead(storageAddress, slot, value, exists, cached);
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

    // TODO: include exists here also - This can for sure come from the trace???
    this.transitionalExecutionResult.noteHashReadRequests.push(new ReadRequest(noteHash, this.trace.accessCounter));

    this.trace.traceNoteHashCheck(storageAddress, noteHash, exists, leafIndex);
    return Promise.resolve(exists);
  }

  /**
   * Write a note hash, trace the write.
   * @param noteHash - the unsiloed note hash to write
   */
  public writeNoteHash(storageAddress: Fr, noteHash: Fr) {
    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    this.transitionalExecutionResult.newNoteHashes.push(new NoteHash(noteHash, this.trace.accessCounter));

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

    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    if (exists) {
      this.transitionalExecutionResult.nullifierReadRequests.push(new ReadRequest(nullifier, this.trace.accessCounter));
    } else {
      this.transitionalExecutionResult.nullifierNonExistentReadRequests.push(
        new ReadRequest(nullifier, this.trace.accessCounter),
      );
    }

    this.trace.traceNullifierCheck(storageAddress, nullifier, exists, isPending, leafIndex);
    return Promise.resolve(exists);
  }

  /**
   * Write a nullifier to the nullifier set, trace the write.
   * @param storageAddress - address of the contract that the nullifier is associated with
   * @param nullifier - the unsiloed nullifier to write
   */
  public async writeNullifier(storageAddress: Fr, nullifier: Fr) {
    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    this.transitionalExecutionResult.newNullifiers.push(
      new Nullifier(nullifier, this.trace.accessCounter, /*noteHash=*/ Fr.ZERO),
    );

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
    const valueAtIndex = await this.hostStorage.commitmentsDb.getL1ToL2LeafValue(msgLeafIndex.toBigInt());
    const exists = valueAtIndex?.equals(msgHash) ?? false;
    this.log.debug(
      `l1ToL2Messages(@${msgLeafIndex}) ?? exists: ${exists}, expected: ${msgHash}, found: ${valueAtIndex}.`,
    );

    this.transitionalExecutionResult.l1ToL2MsgReadRequests.push(new ReadRequest(msgHash, this.trace.accessCounter));

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
    const message = new L2ToL1Message(recipientAddress, content, 0);
    this.newL1Messages.push(message);

    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    this.transitionalExecutionResult.newL2ToL1Messages.push(message);
  }

  public writeLog(contractAddress: Fr, event: Fr, log: Fr[]) {
    this.log.debug(`UnencryptedL2Log(${contractAddress}) += event ${event} with ${log.length} fields.`);
    const ulog = new UnencryptedL2Log(
      AztecAddress.fromField(contractAddress),
      EventSelector.fromField(event),
      Buffer.concat(log.map(f => f.toBuffer())),
    );
    const logHash = Fr.fromBuffer(ulog.hash());

    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    this.transitionalExecutionResult.unencryptedLogs.push(ulog);
    this.transitionalExecutionResult.allUnencryptedLogs.push(ulog);
    // this duplicates exactly what happens in the trace just for the purpose of transitional integration with the kernel
    this.transitionalExecutionResult.unencryptedLogsHashes.push(
      // TODO(6578): explain magic number 4 here
      new LogHash(logHash, this.trace.accessCounter, new Fr(ulog.length + 4)),
    );
    // TODO(6206): likely need to track this here and not just in the transitional logic.

    // TODO(6205): why are logs pushed here but logs hashes are traced?
    this.newLogs.push(ulog);
    this.trace.traceNewLog(logHash);
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
    this.newL1Messages.push(...nestedJournal.newL1Messages);
    this.newLogs.push(...nestedJournal.newLogs);

    // TRANSITIONAL: This should be removed once the kernel handles and entire enqueued call per circuit
    this.transitionalExecutionResult.allUnencryptedLogs.push(
      ...nestedJournal.transitionalExecutionResult.allUnencryptedLogs,
    );
  }

  /**
   * Reject nested world state, merging in its trace, but not accepting any state modifications
   */
  public rejectNestedCallState(nestedJournal: AvmPersistableStateManager) {
    // Merge World State Access Trace
    this.trace.acceptAndMerge(nestedJournal.trace);
  }

  // TODO:(5818): do we need this type anymore?
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
      sideEffectCounter: this.trace.accessCounter,
    };
  }
}
