import { type FunctionData, PrivateCallStackItem, PrivateCircuitPublicInputs } from '@aztec/circuits.js';
import { type FunctionArtifactWithDebugMetadata } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';

import { witnessMapToFields } from '../acvm/deserialize.js';
import { Oracle, acvm, extractCallStack } from '../acvm/index.js';
import { ExecutionError } from '../common/errors.js';
import { type ClientExecutionContext } from './client_execution_context.js';
import { type ExecutionResult } from './execution_result.js';
import { AcirSimulator } from './simulator.js';

/**
 * Execute a private function and return the execution result.
 */
export async function executePrivateFunction(
  context: ClientExecutionContext,
  artifact: FunctionArtifactWithDebugMetadata,
  contractAddress: AztecAddress,
  functionData: FunctionData,
  log = createDebugLogger('aztec:simulator:secret_execution'),
): Promise<ExecutionResult> {
  const functionSelector = functionData.selector;
  log.verbose(`Executing external function ${contractAddress}:${functionSelector}(${artifact.name})`);
  const acir = artifact.bytecode;
  const initialWitness = context.getInitialWitness(artifact);
  const acvmCallback = new Oracle(context);
  const acirExecutionResult = await acvm(await AcirSimulator.getSolver(), acir, initialWitness, acvmCallback).catch(
    (err: Error) => {
      throw new ExecutionError(
        err.message,
        {
          contractAddress,
          functionSelector,
        },
        extractCallStack(err, artifact.debug),
        { cause: err },
      );
    },
  );
  const partialWitness = acirExecutionResult.partialWitness;
  const returnWitness = witnessMapToFields(acirExecutionResult.returnWitness);
  const publicInputs = PrivateCircuitPublicInputs.fromFields(returnWitness);

  const encryptedLogs = context.getEncryptedLogs();
  const unencryptedLogs = context.getUnencryptedLogs();

  const callStackItem = new PrivateCallStackItem(contractAddress, functionData, publicInputs);

  const rawReturnValues = await context.unpackReturns(publicInputs.returnsHash);

  const noteHashLeafIndexMap = context.getNoteHashLeafIndexMap();
  const newNotes = context.getNewNotes();
  const nullifiedNoteHashCounters = context.getNullifiedNoteHashCounters();
  const nestedExecutions = context.getNestedExecutions();
  const enqueuedPublicFunctionCalls = context.getEnqueuedPublicFunctionCalls();

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
    encryptedLogs,
    unencryptedLogs,
  };
}
