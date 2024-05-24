import { type CircuitWitnessGenerationStats } from '@aztec/circuit-types/stats';
import { Fr, FunctionData, PrivateCallStackItem, PrivateCircuitPublicInputs } from '@aztec/circuits.js';
import type { FunctionArtifact, FunctionSelector } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';

import { witnessMapToFields } from '../acvm/deserialize.js';
import { Oracle, acvm, extractCallStack } from '../acvm/index.js';
import { ExecutionError } from '../common/errors.js';
import { type ClientExecutionContext } from './client_execution_context.js';
import { type ExecutionResult } from './execution_result.js';

/**
 * Execute a private function and return the execution result.
 */
export async function executePrivateFunction(
  context: ClientExecutionContext,
  artifact: FunctionArtifact,
  contractAddress: AztecAddress,
  functionSelector: FunctionSelector,
  log = createDebugLogger('aztec:simulator:secret_execution'),
): Promise<ExecutionResult> {
  const functionName = await context.getDebugFunctionName();
  log.verbose(`Executing external function ${contractAddress}:${functionSelector}(${functionName})`);
  const acir = artifact.bytecode;
  const initialWitness = context.getInitialWitness(artifact);
  const acvmCallback = new Oracle(context);
  const timer = new Timer();
  const acirExecutionResult = await acvm(acir, initialWitness, acvmCallback).catch((err: Error) => {
    throw new ExecutionError(
      err.message,
      {
        contractAddress,
        functionSelector,
      },
      extractCallStack(err, artifact.debug),
      { cause: err },
    );
  });
  const duration = timer.ms();
  const partialWitness = acirExecutionResult.partialWitness;
  const returnWitness = witnessMapToFields(acirExecutionResult.returnWitness);
  const publicInputs = PrivateCircuitPublicInputs.fromFields(returnWitness);

  // TODO (alexg) estimate this size
  const initialWitnessSize = witnessMapToFields(initialWitness).length * Fr.SIZE_IN_BYTES;
  log.debug(`Ran external function ${contractAddress.toString()}:${functionSelector}`, {
    circuitName: 'app-circuit',
    duration,
    eventName: 'circuit-witness-generation',
    inputSize: initialWitnessSize,
    outputSize: publicInputs.toBuffer().length,
    appCircuitName: functionName,
  } satisfies CircuitWitnessGenerationStats);

  context.chopNoteEncryptedLogs();
  const noteEncryptedLogs = context.getNoteEncryptedLogs();
  const encryptedLogs = context.getEncryptedLogs();
  const unencryptedLogs = context.getUnencryptedLogs();

  const callStackItem = new PrivateCallStackItem(
    contractAddress,
    new FunctionData(functionSelector, true),
    publicInputs,
  );

  const rawReturnValues = await context.unpackReturns(publicInputs.returnsHash);

  const noteHashLeafIndexMap = context.getNoteHashLeafIndexMap();
  const newNotes = context.getNewNotes();
  const nullifiedNoteHashCounters = context.getNullifiedNoteHashCounters();
  const nestedExecutions = context.getNestedExecutions();
  const enqueuedPublicFunctionCalls = context.getEnqueuedPublicFunctionCalls();
  const publicTeardownFunctionCall = context.getPublicTeardownFunctionCall();

  log.debug(`Returning from call to ${contractAddress.toString()}:${functionSelector}`);

  return {
    acir,
    partialWitness,
    callStackItem,
    returnValues: rawReturnValues,
    noteHashLeafIndexMap,
    newNotes,
    nullifiedNoteHashCounters,
    vk: Buffer.from(artifact.verificationKey!, 'hex'),
    nestedExecutions,
    enqueuedPublicFunctionCalls,
    noteEncryptedLogs,
    publicTeardownFunctionCall,
    encryptedLogs,
    unencryptedLogs,
  };
}
