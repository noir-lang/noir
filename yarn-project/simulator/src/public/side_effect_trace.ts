import { UnencryptedFunctionL2Logs, UnencryptedL2Log } from '@aztec/circuit-types';
import {
  AvmContractInstanceHint,
  AvmExecutionHints,
  AvmExternalCallHint,
  AvmKeyValueHint,
  AztecAddress,
  CallContext,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  EthAddress,
  Gas,
  L2ToL1Message,
  LogHash,
  NoteHash,
  Nullifier,
  ReadRequest,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { type AvmContractCallResult } from '../avm/avm_contract_call_result.js';
import { type AvmExecutionEnvironment } from '../avm/avm_execution_environment.js';
import { createSimulationError } from '../common/errors.js';
import { type PublicExecutionRequest, type PublicExecutionResult } from './execution.js';
import { type PublicSideEffectTraceInterface } from './side_effect_trace_interface.js';

export type TracedContractInstance = { exists: boolean } & ContractInstanceWithAddress;

export class PublicSideEffectTrace implements PublicSideEffectTraceInterface {
  public logger = createDebugLogger('aztec:public_side_effect_trace');

  /** The side effect counter increments with every call to the trace. */
  private sideEffectCounter: number; // kept as number until finalized for efficiency

  private contractStorageReads: ContractStorageRead[] = [];
  private contractStorageUpdateRequests: ContractStorageUpdateRequest[] = [];

  private noteHashReadRequests: ReadRequest[] = [];
  private noteHashes: NoteHash[] = [];

  private nullifierReadRequests: ReadRequest[] = [];
  private nullifierNonExistentReadRequests: ReadRequest[] = [];
  private nullifiers: Nullifier[] = [];

  private l1ToL2MsgReadRequests: ReadRequest[] = [];
  private newL2ToL1Messages: L2ToL1Message[] = [];

  private unencryptedLogs: UnencryptedL2Log[] = [];
  private allUnencryptedLogs: UnencryptedL2Log[] = [];
  private unencryptedLogsHashes: LogHash[] = [];

  private gotContractInstances: ContractInstanceWithAddress[] = [];

  private nestedExecutions: PublicExecutionResult[] = [];

  private avmCircuitHints: AvmExecutionHints;

  constructor(
    /** The counter of this trace's first side effect. */
    public readonly startSideEffectCounter: number = 0,
  ) {
    this.sideEffectCounter = startSideEffectCounter;
    this.avmCircuitHints = AvmExecutionHints.empty();
  }

  public fork() {
    return new PublicSideEffectTrace(this.sideEffectCounter);
  }

  public getCounter() {
    return this.sideEffectCounter;
  }

  private incrementSideEffectCounter() {
    this.sideEffectCounter++;
  }

  public tracePublicStorageRead(storageAddress: Fr, slot: Fr, value: Fr, _exists: boolean, _cached: boolean) {
    // TODO(4805): check if some threshold is reached for max storage reads
    // (need access to parent length, or trace needs to be initialized with parent's contents)
    // NOTE: exists and cached are unused for now but may be used for optimizations or kernel hints later
    this.contractStorageReads.push(
      new ContractStorageRead(slot, value, this.sideEffectCounter, AztecAddress.fromField(storageAddress)),
    );
    this.avmCircuitHints.storageValues.items.push(
      new AvmKeyValueHint(/*key=*/ new Fr(this.sideEffectCounter), /*value=*/ value),
    );
    this.logger.debug(`SLOAD cnt: ${this.sideEffectCounter} val: ${value} slot: ${slot}`);
    this.incrementSideEffectCounter();
  }

  public tracePublicStorageWrite(storageAddress: Fr, slot: Fr, value: Fr) {
    // TODO(4805): check if some threshold is reached for max storage writes
    // (need access to parent length, or trace needs to be initialized with parent's contents)
    this.contractStorageUpdateRequests.push(
      new ContractStorageUpdateRequest(slot, value, this.sideEffectCounter, storageAddress),
    );
    this.logger.debug(`SSTORE cnt: ${this.sideEffectCounter} val: ${value} slot: ${slot}`);
    this.incrementSideEffectCounter();
  }

  public traceNoteHashCheck(_storageAddress: Fr, noteHash: Fr, _leafIndex: Fr, exists: boolean) {
    // TODO(4805): check if some threshold is reached for max note hash checks
    // NOTE: storageAddress is unused but will be important when an AVM circuit processes an entire enqueued call
    // TODO(dbanks12): leafIndex is unused for now but later must be used by kernel to constrain that the kernel
    // is in fact checking the leaf indicated by the user
    this.noteHashReadRequests.push(new ReadRequest(noteHash, this.sideEffectCounter));
    this.avmCircuitHints.noteHashExists.items.push(
      new AvmKeyValueHint(/*key=*/ new Fr(this.sideEffectCounter), /*value=*/ new Fr(exists ? 1 : 0)),
    );
    this.logger.debug(`NOTE_HASH_CHECK cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  public traceNewNoteHash(_storageAddress: Fr, noteHash: Fr) {
    // TODO(4805): check if some threshold is reached for max new note hash
    // NOTE: storageAddress is unused but will be important when an AVM circuit processes an entire enqueued call
    // TODO(dbanks12): non-existent note hashes should emit a read request of the note hash that actually
    // IS there, and the AVM circuit should accept THAT noteHash as a hint. The circuit will then compare
    // the noteHash against the one provided by the user code to determine what to return to the user (exists or not),
    // and will then propagate the actually-present noteHash to its public inputs.
    this.noteHashes.push(new NoteHash(noteHash, this.sideEffectCounter));
    this.logger.debug(`NEW_NOTE_HASH cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  public traceNullifierCheck(_storageAddress: Fr, nullifier: Fr, _leafIndex: Fr, exists: boolean, _isPending: boolean) {
    // TODO(4805): check if some threshold is reached for max new nullifier
    // NOTE: storageAddress is unused but will be important when an AVM circuit processes an entire enqueued call
    // NOTE: isPending and leafIndex are unused for now but may be used for optimizations or kernel hints later
    const readRequest = new ReadRequest(nullifier, this.sideEffectCounter);
    if (exists) {
      this.nullifierReadRequests.push(readRequest);
    } else {
      this.nullifierNonExistentReadRequests.push(readRequest);
    }
    this.avmCircuitHints.nullifierExists.items.push(
      new AvmKeyValueHint(/*key=*/ new Fr(this.sideEffectCounter), /*value=*/ new Fr(exists ? 1 : 0)),
    );
    this.logger.debug(`NULLIFIER_EXISTS cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  public traceNewNullifier(_storageAddress: Fr, nullifier: Fr) {
    // TODO(4805): check if some threshold is reached for max new nullifier
    // NOTE: storageAddress is unused but will be important when an AVM circuit processes an entire enqueued call
    this.nullifiers.push(new Nullifier(nullifier, this.sideEffectCounter, /*noteHash=*/ Fr.ZERO));
    this.logger.debug(`NEW_NULLIFIER cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  public traceL1ToL2MessageCheck(_contractAddress: Fr, msgHash: Fr, _msgLeafIndex: Fr, exists: boolean) {
    // TODO(4805): check if some threshold is reached for max message reads
    // NOTE: contractAddress is unused but will be important when an AVM circuit processes an entire enqueued call
    // TODO(dbanks12): leafIndex is unused for now but later must be used by kernel to constrain that the kernel
    // is in fact checking the leaf indicated by the user
    this.l1ToL2MsgReadRequests.push(new ReadRequest(msgHash, this.sideEffectCounter));
    this.avmCircuitHints.l1ToL2MessageExists.items.push(
      new AvmKeyValueHint(/*key=*/ new Fr(this.sideEffectCounter), /*value=*/ new Fr(exists ? 1 : 0)),
    );
    this.logger.debug(`L1_TO_L2_MSG_CHECK cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  public traceNewL2ToL1Message(recipient: Fr, content: Fr) {
    // TODO(4805): check if some threshold is reached for max messages
    const recipientAddress = EthAddress.fromField(recipient);
    this.newL2ToL1Messages.push(new L2ToL1Message(recipientAddress, content, this.sideEffectCounter));
    this.logger.debug(`NEW_L2_TO_L1_MSG cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  public traceUnencryptedLog(contractAddress: Fr, log: Fr[]) {
    // TODO(4805): check if some threshold is reached for max logs
    const ulog = new UnencryptedL2Log(
      AztecAddress.fromField(contractAddress),
      Buffer.concat(log.map(f => f.toBuffer())),
    );
    const basicLogHash = Fr.fromBuffer(ulog.hash());
    this.unencryptedLogs.push(ulog);
    this.allUnencryptedLogs.push(ulog);
    // We want the length of the buffer output from function_l2_logs -> toBuffer to equal the stored log length in the kernels.
    // The kernels store the length of the processed log as 4 bytes; thus for this length value to match the log length stored in the kernels,
    // we need to add four to the length here.
    // https://github.com/AztecProtocol/aztec-packages/issues/6578#issuecomment-2125003435
    this.unencryptedLogsHashes.push(new LogHash(basicLogHash, this.sideEffectCounter, new Fr(ulog.length + 4)));
    this.logger.debug(`NEW_UNENCRYPTED_LOG cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  public traceGetContractInstance(instance: TracedContractInstance) {
    // TODO(4805): check if some threshold is reached for max contract instance retrievals
    this.gotContractInstances.push(instance);
    this.avmCircuitHints.contractInstances.items.push(
      new AvmContractInstanceHint(
        instance.address,
        new Fr(instance.exists ? 1 : 0),
        instance.salt,
        instance.deployer,
        instance.contractClassId,
        instance.initializationHash,
        instance.publicKeysHash,
      ),
    );
    this.logger.debug(`CONTRACT_INSTANCE cnt: ${this.sideEffectCounter}`);
    this.incrementSideEffectCounter();
  }

  /**
   * Trace a nested call.
   * Accept some results from a finished nested call's trace into this one.
   */
  public traceNestedCall(
    /** The trace of the nested call. */
    nestedCallTrace: PublicSideEffectTrace,
    /** The execution environment of the nested call. */
    nestedEnvironment: AvmExecutionEnvironment,
    /** How much gas was available for this public execution. */
    startGasLeft: Gas,
    /** How much gas was left after this public execution. */
    endGasLeft: Gas,
    /** Bytecode used for this execution. */
    bytecode: Buffer,
    /** The call's results */
    avmCallResults: AvmContractCallResult,
    /** Function name for logging */
    functionName: string = 'unknown',
  ) {
    const result = nestedCallTrace.toPublicExecutionResult(
      nestedEnvironment,
      startGasLeft,
      endGasLeft,
      bytecode,
      avmCallResults,
      functionName,
    );
    this.sideEffectCounter = result.endSideEffectCounter.toNumber();
    // when a nested call returns, caller accepts its updated counter
    this.allUnencryptedLogs.push(...result.allUnencryptedLogs.logs);
    // NOTE: eventually if the AVM circuit processes an entire enqueued call,
    // this function will accept all of the nested's side effects into this instance
    this.nestedExecutions.push(result);

    const gasUsed = new Gas(
      result.startGasLeft.daGas - result.endGasLeft.daGas,
      result.startGasLeft.l2Gas - result.endGasLeft.l2Gas,
    );
    this.avmCircuitHints.externalCalls.items.push(
      new AvmExternalCallHint(
        /*success=*/ new Fr(result.reverted ? 0 : 1),
        result.returnValues,
        gasUsed,
        result.endSideEffectCounter,
      ),
    );
  }

  /**
   * Convert this trace to a PublicExecutionResult for use externally to the simulator.
   */
  public toPublicExecutionResult(
    /** The execution environment of the nested call. */
    avmEnvironment: AvmExecutionEnvironment,
    /** How much gas was available for this public execution. */
    startGasLeft: Gas,
    /** How much gas was left after this public execution. */
    endGasLeft: Gas,
    /** Bytecode used for this execution. */
    bytecode: Buffer,
    /** The call's results */
    avmCallResults: AvmContractCallResult,
    /** Function name for logging */
    functionName: string = 'unknown',
  ): PublicExecutionResult {
    return {
      executionRequest: createPublicExecutionRequest(avmEnvironment),

      startSideEffectCounter: new Fr(this.startSideEffectCounter),
      endSideEffectCounter: new Fr(this.sideEffectCounter),
      startGasLeft,
      endGasLeft,
      transactionFee: avmEnvironment.transactionFee,

      bytecode,
      calldata: avmEnvironment.calldata,
      returnValues: avmCallResults.output,
      reverted: avmCallResults.reverted,
      revertReason: avmCallResults.revertReason ? createSimulationError(avmCallResults.revertReason) : undefined,

      contractStorageReads: this.contractStorageReads,
      contractStorageUpdateRequests: this.contractStorageUpdateRequests,
      noteHashReadRequests: this.noteHashReadRequests,
      noteHashes: this.noteHashes,
      nullifierReadRequests: this.nullifierReadRequests,
      nullifierNonExistentReadRequests: this.nullifierNonExistentReadRequests,
      nullifiers: this.nullifiers,
      l1ToL2MsgReadRequests: this.l1ToL2MsgReadRequests,
      l2ToL1Messages: this.newL2ToL1Messages,
      // correct the type on these now that they are finalized (lists won't grow)
      unencryptedLogs: new UnencryptedFunctionL2Logs(this.unencryptedLogs),
      allUnencryptedLogs: new UnencryptedFunctionL2Logs(this.allUnencryptedLogs),
      unencryptedLogsHashes: this.unencryptedLogsHashes,
      // TODO(dbanks12): process contract instance read requests in public kernel
      //gotContractInstances: this.gotContractInstances,

      nestedExecutions: this.nestedExecutions,

      avmCircuitHints: this.avmCircuitHints,

      functionName,
    };
  }
}

/**
 * Helper function to create a public execution request from an AVM execution environment
 */
function createPublicExecutionRequest(avmEnvironment: AvmExecutionEnvironment): PublicExecutionRequest {
  const callContext = CallContext.from({
    msgSender: avmEnvironment.sender,
    storageContractAddress: avmEnvironment.storageAddress,
    functionSelector: avmEnvironment.functionSelector,
    isDelegateCall: avmEnvironment.isDelegateCall,
    isStaticCall: avmEnvironment.isStaticCall,
  });
  return {
    contractAddress: avmEnvironment.address,
    functionSelector: avmEnvironment.functionSelector,
    callContext,
    // execution request does not contain AvmContextInputs prefix
    args: avmEnvironment.getCalldataWithoutPrefix(),
  };
}
