import { type Gas } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

import { type AvmExecutionEnvironment } from '../avm/avm_execution_environment.js';
import { type AvmContractCallResults } from '../avm/avm_message_call_result.js';
import { type TracedContractInstance } from './side_effect_trace.js';

export interface PublicSideEffectTraceInterface {
  fork(): PublicSideEffectTraceInterface;
  getCounter(): number;
  tracePublicStorageRead(storageAddress: Fr, slot: Fr, value: Fr, exists: boolean, cached: boolean): void;
  tracePublicStorageWrite(storageAddress: Fr, slot: Fr, value: Fr): void;
  traceNoteHashCheck(storageAddress: Fr, noteHash: Fr, leafIndex: Fr, exists: boolean): void;
  traceNewNoteHash(storageAddress: Fr, noteHash: Fr): void;
  traceNullifierCheck(storageAddress: Fr, nullifier: Fr, leafIndex: Fr, exists: boolean, isPending: boolean): void;
  traceNewNullifier(storageAddress: Fr, nullifier: Fr): void;
  traceL1ToL2MessageCheck(contractAddress: Fr, msgHash: Fr, msgLeafIndex: Fr, exists: boolean): void;
  // TODO(dbanks12): should new message accept contract address as arg?
  traceNewL2ToL1Message(recipient: Fr, content: Fr): void;
  traceUnencryptedLog(contractAddress: Fr, log: Fr[]): void;
  // TODO(dbanks12): odd that getContractInstance is a one-off in that it accepts an entire object instead of components
  traceGetContractInstance(instance: TracedContractInstance): void;
  traceNestedCall(
    /** The trace of the nested call. */
    nestedCallTrace: PublicSideEffectTraceInterface,
    /** The execution environment of the nested call. */
    nestedEnvironment: AvmExecutionEnvironment,
    /** How much gas was available for this public execution. */
    // TODO(dbanks12): consider moving to AvmExecutionEnvironment
    startGasLeft: Gas,
    /** How much gas was left after this public execution. */
    // TODO(dbanks12): consider moving to AvmContractCallResults
    endGasLeft: Gas,
    /** Bytecode used for this execution. */
    bytecode: Buffer,
    /** The call's results */
    avmCallResults: AvmContractCallResults,
    /** Function name */
    functionName: string,
  ): void;
}
