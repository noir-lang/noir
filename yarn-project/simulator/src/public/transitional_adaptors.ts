// All code in this file needs to die once the public executor is phased out in favor of the AVM.
import { UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  AvmExecutionHints,
  AvmExternalCallHint,
  AvmKeyValueHint,
  CallContext,
  Gas,
  type GasSettings,
  type GlobalVariables,
  type Header,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { promisify } from 'util';
import { gunzip } from 'zlib';

import { type AvmContext } from '../avm/avm_context.js';
import { AvmExecutionEnvironment } from '../avm/avm_execution_environment.js';
import { type AvmContractCallResults } from '../avm/avm_message_call_result.js';
import { type PartialPublicExecutionResult } from '../avm/journal/journal.js';
import { type WorldStateAccessTrace } from '../avm/journal/trace.js';
import { Mov } from '../avm/opcodes/memory.js';
import { createSimulationError } from '../common/errors.js';
import { type PublicExecution, type PublicExecutionResult } from './execution.js';

/**
 * Convert a PublicExecution(Environment) object to an AvmExecutionEnvironment
 *
 * @param current
 * @param globalVariables
 * @returns
 */
export function createAvmExecutionEnvironment(
  current: PublicExecution,
  header: Header,
  globalVariables: GlobalVariables,
  gasSettings: GasSettings,
  transactionFee: Fr,
): AvmExecutionEnvironment {
  return new AvmExecutionEnvironment(
    current.contractAddress,
    current.callContext.storageContractAddress,
    current.callContext.msgSender,
    globalVariables.gasFees.feePerL2Gas,
    globalVariables.gasFees.feePerDaGas,
    /*contractCallDepth=*/ Fr.zero(),
    header,
    globalVariables,
    current.callContext.isStaticCall,
    current.callContext.isDelegateCall,
    current.args,
    gasSettings,
    transactionFee,
    current.functionSelector,
  );
}

export function createPublicExecution(
  startSideEffectCounter: number,
  avmEnvironment: AvmExecutionEnvironment,
  calldata: Fr[],
): PublicExecution {
  const callContext = CallContext.from({
    msgSender: avmEnvironment.sender,
    storageContractAddress: avmEnvironment.storageAddress,
    functionSelector: avmEnvironment.temporaryFunctionSelector,
    isDelegateCall: avmEnvironment.isDelegateCall,
    isStaticCall: avmEnvironment.isStaticCall,
    sideEffectCounter: startSideEffectCounter,
  });
  const execution: PublicExecution = {
    contractAddress: avmEnvironment.address,
    callContext,
    args: calldata,
    functionSelector: avmEnvironment.temporaryFunctionSelector,
  };
  return execution;
}

function computeHints(trace: WorldStateAccessTrace, executionResult: PartialPublicExecutionResult): AvmExecutionHints {
  return new AvmExecutionHints(
    trace.publicStorageReads.map(read => new AvmKeyValueHint(read.counter, read.value)),
    trace.noteHashChecks.map(check => new AvmKeyValueHint(check.counter, new Fr(check.exists ? 1 : 0))),
    trace.nullifierChecks.map(check => new AvmKeyValueHint(check.counter, new Fr(check.exists ? 1 : 0))),
    trace.l1ToL2MessageChecks.map(check => new AvmKeyValueHint(check.counter, new Fr(check.exists ? 1 : 0))),
    executionResult.nestedExecutions.map(nested => {
      const gasUsed = new Gas(
        nested.startGasLeft.daGas - nested.endGasLeft.daGas,
        nested.startGasLeft.l2Gas - nested.endGasLeft.l2Gas,
      );
      return new AvmExternalCallHint(/*success=*/ new Fr(nested.reverted ? 0 : 1), nested.returnValues, gasUsed);
    }),
  );
}

export function convertAvmResultsToPxResult(
  avmResult: AvmContractCallResults,
  startSideEffectCounter: number,
  fromPx: PublicExecution,
  startGas: Gas,
  endAvmContext: AvmContext,
  bytecode: Buffer | undefined,
): PublicExecutionResult {
  const endPersistableState = endAvmContext.persistableState;
  const endMachineState = endAvmContext.machineState;

  return {
    ...endPersistableState.transitionalExecutionResult, // includes nestedExecutions
    execution: fromPx,
    returnValues: avmResult.output,
    startSideEffectCounter: new Fr(startSideEffectCounter),
    endSideEffectCounter: new Fr(endPersistableState.trace.accessCounter),
    unencryptedLogs: new UnencryptedFunctionL2Logs(endPersistableState.transitionalExecutionResult.unencryptedLogs),
    allUnencryptedLogs: new UnencryptedFunctionL2Logs(
      endPersistableState.transitionalExecutionResult.allUnencryptedLogs,
    ),
    reverted: avmResult.reverted,
    revertReason: avmResult.revertReason ? createSimulationError(avmResult.revertReason) : undefined,
    startGasLeft: startGas,
    endGasLeft: endMachineState.gasLeft,
    transactionFee: endAvmContext.environment.transactionFee,
    bytecode: bytecode,
    calldata: endAvmContext.environment.calldata,
    avmHints: computeHints(endPersistableState.trace, endPersistableState.transitionalExecutionResult),
  };
}

const AVM_MAGIC_SUFFIX = Buffer.from([
  Mov.opcode, // opcode
  0x00, // indirect
  ...Buffer.from('000018ca', 'hex'), // srcOffset
  ...Buffer.from('000018ca', 'hex'), // dstOffset
]);

export function markBytecodeAsAvm(bytecode: Buffer): Buffer {
  return Buffer.concat([bytecode, AVM_MAGIC_SUFFIX]);
}

// This is just a helper function for the AVM circuit.
export async function decompressBytecodeIfCompressed(bytecode: Buffer): Promise<Buffer> {
  try {
    return await promisify(gunzip)(bytecode);
  } catch {
    // If the bytecode is not compressed, the gunzip call will throw an error
    // In this case, we assume the bytecode is not compressed and continue.
    return Promise.resolve(bytecode);
  }
}

export async function isAvmBytecode(bytecode: Buffer): Promise<boolean> {
  const decompressedBytecode = await decompressBytecodeIfCompressed(bytecode);
  const magicSize = AVM_MAGIC_SUFFIX.length;
  return decompressedBytecode.subarray(-magicSize).equals(AVM_MAGIC_SUFFIX);
}
