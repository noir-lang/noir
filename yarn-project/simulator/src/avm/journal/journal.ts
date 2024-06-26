import { AztecAddress, type FunctionSelector, type Gas } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { SerializableContractInstance } from '@aztec/types/contracts';

import { type TracedContractInstance } from '../../public/side_effect_trace.js';
import { type PublicSideEffectTraceInterface } from '../../public/side_effect_trace_interface.js';
import { type AvmExecutionEnvironment } from '../avm_execution_environment.js';
import { type AvmContractCallResults } from '../avm_message_call_result.js';
import { type HostStorage } from './host_storage.js';
import { NullifierManager } from './nullifiers.js';
import { PublicStorage } from './public_storage.js';

/**
 * A class to manage persistable AVM state for contract calls.
 * Maintains a cache of the current world state,
 * a trace of all side effects.
 *
 * The simulator should make any world state / tree queries through this object.
 *
 * Manages merging of successful/reverted child state into current state.
 */
export class AvmPersistableStateManager {
  private readonly log: DebugLogger = createDebugLogger('aztec:avm_simulator:state_manager');

  constructor(
    /** Reference to node storage */
    private hostStorage: HostStorage,
    /** Side effect trace */
    private trace: PublicSideEffectTraceInterface,
    /** Public storage, including cached writes */
    public readonly publicStorage: PublicStorage,
    /** Nullifier set, including cached/recently-emitted nullifiers */
    private readonly nullifiers: NullifierManager,
  ) {}

  /**
   * Create a new state manager with some preloaded pending siloed nullifiers
   */
  public static newWithPendingSiloedNullifiers(
    hostStorage: HostStorage,
    trace: PublicSideEffectTraceInterface,
    pendingSiloedNullifiers: Fr[],
  ) {
    const parentNullifiers = NullifierManager.newWithPendingSiloedNullifiers(
      hostStorage.commitmentsDb,
      pendingSiloedNullifiers,
    );
    return new AvmPersistableStateManager(
      hostStorage,
      trace,
      /*publicStorage=*/ new PublicStorage(hostStorage.publicStateDb),
      /*nullifiers=*/ parentNullifiers.fork(),
    );
  }

  /**
   * Create a new state manager forked from this one
   */
  public fork() {
    return new AvmPersistableStateManager(
      this.hostStorage,
      this.trace.fork(),
      this.publicStorage.fork(),
      this.nullifiers.fork(),
    );
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
    this.trace.tracePublicStorageRead(storageAddress, slot, value, exists, cached);
    return Promise.resolve(value);
  }

  /**
   * Read from public storage, don't trace the read.
   *
   * @param storageAddress - the address of the contract whose storage is being read from
   * @param slot - the slot in the contract's storage being read from
   * @returns the latest value written to slot, or 0 if never written to before
   */
  public async peekStorage(storageAddress: Fr, slot: Fr): Promise<Fr> {
    const { value, exists, cached } = await this.publicStorage.read(storageAddress, slot);
    this.log.debug(
      `Storage peek  (address=${storageAddress}, slot=${slot}): value=${value}, exists=${exists}, cached=${cached}`,
    );
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
    this.trace.traceNoteHashCheck(storageAddress, noteHash, leafIndex, exists);
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
      `nullifiers(${storageAddress})@${nullifier} ?? leafIndex: ${leafIndex}, exists: ${exists}, pending: ${isPending}.`,
    );
    this.trace.traceNullifierCheck(storageAddress, nullifier, leafIndex, exists, isPending);
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
  public async checkL1ToL2MessageExists(contractAddress: Fr, msgHash: Fr, msgLeafIndex: Fr): Promise<boolean> {
    const valueAtIndex = await this.hostStorage.commitmentsDb.getL1ToL2LeafValue(msgLeafIndex.toBigInt());
    const exists = valueAtIndex?.equals(msgHash) ?? false;
    this.log.debug(
      `l1ToL2Messages(@${msgLeafIndex}) ?? exists: ${exists}, expected: ${msgHash}, found: ${valueAtIndex}.`,
    );
    this.trace.traceL1ToL2MessageCheck(contractAddress, msgHash, msgLeafIndex, exists);
    return Promise.resolve(exists);
  }

  /**
   * Write an L2 to L1 message.
   * @param recipient - L1 contract address to send the message to.
   * @param content - Message content.
   */
  public writeL2ToL1Message(recipient: Fr, content: Fr) {
    this.log.debug(`L1Messages(${recipient}) += ${content}.`);
    this.trace.traceNewL2ToL1Message(recipient, content);
  }

  /**
   * Write an unencrypted log
   * @param contractAddress - address of the contract that emitted the log
   * @param event - log event selector
   * @param log - log contents
   */
  public writeUnencryptedLog(contractAddress: Fr, log: Fr[]) {
    this.log.debug(`UnencryptedL2Log(${contractAddress}) += event with ${log.length} fields.`);
    this.trace.traceUnencryptedLog(contractAddress, log);
  }

  /**
   * Get a contract instance.
   * @param contractAddress - address of the contract instance to retrieve.
   * @returns the contract instance with an "exists" flag
   */
  public async getContractInstance(contractAddress: Fr): Promise<TracedContractInstance> {
    let exists = true;
    const aztecAddress = AztecAddress.fromField(contractAddress);
    let instance = await this.hostStorage.contractsDb.getContractInstance(aztecAddress);
    if (instance === undefined) {
      instance = SerializableContractInstance.empty().withAddress(aztecAddress);
      exists = false;
    }
    this.log.debug(
      `Get Contract instance (address=${contractAddress}): exists=${exists}, instance=${JSON.stringify(instance)}`,
    );
    const tracedInstance = { ...instance, exists };
    this.trace.traceGetContractInstance(tracedInstance);
    return Promise.resolve(tracedInstance);
  }

  /**
   * Accept nested world state modifications
   */
  public acceptNestedCallState(nestedState: AvmPersistableStateManager) {
    this.publicStorage.acceptAndMerge(nestedState.publicStorage);
    this.nullifiers.acceptAndMerge(nestedState.nullifiers);
  }

  /**
   * Get a contract's bytecode from the contracts DB
   */
  public async getBytecode(contractAddress: AztecAddress, selector: FunctionSelector): Promise<Buffer | undefined> {
    return await this.hostStorage.contractsDb.getBytecode(contractAddress, selector);
  }

  /**
   * Accept the nested call's state and trace the nested call
   */
  public async processNestedCall(
    nestedState: AvmPersistableStateManager,
    success: boolean,
    nestedEnvironment: AvmExecutionEnvironment,
    startGasLeft: Gas,
    endGasLeft: Gas,
    bytecode: Buffer,
    avmCallResults: AvmContractCallResults,
  ) {
    if (success) {
      this.acceptNestedCallState(nestedState);
    }
    const functionName =
      (await nestedState.hostStorage.contractsDb.getDebugFunctionName(
        nestedEnvironment.address,
        nestedEnvironment.temporaryFunctionSelector,
      )) ?? `${nestedEnvironment.address}:${nestedEnvironment.temporaryFunctionSelector}`;
    this.log.verbose(`[AVM] Calling nested function ${functionName}`);
    this.trace.traceNestedCall(
      nestedState.trace,
      nestedEnvironment,
      startGasLeft,
      endGasLeft,
      bytecode,
      avmCallResults,
      functionName,
    );
  }
}
