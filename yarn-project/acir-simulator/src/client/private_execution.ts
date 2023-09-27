import { FunctionData, PrivateCallStackItem } from '@aztec/circuits.js';
import { decodeReturnValues } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { to2Fields } from '@aztec/foundation/serialize';

import { extractPrivateCircuitPublicInputs } from '../acvm/deserialize.js';
import { Oracle, acvm, extractCallStack } from '../acvm/index.js';
import { ExecutionError } from '../common/errors.js';
import { ClientExecutionContext } from './client_execution_context.js';
import { FunctionAbiWithDebugMetadata } from './db_oracle.js';
import { ExecutionResult } from './execution_result.js';
import { AcirSimulator } from './simulator.js';

/**
 * Execute a private function and return the execution result.
 */
export async function executePrivateFunction(
  context: ClientExecutionContext,
  abi: FunctionAbiWithDebugMetadata,
  contractAddress: AztecAddress,
  functionData: FunctionData,
  log = createDebugLogger('aztec:simulator:secret_execution'),
): Promise<ExecutionResult> {
  const functionSelector = functionData.selector;
  log(`Executing external function ${contractAddress}:${functionSelector}`);

  const acir = Buffer.from(abi.bytecode, 'base64');
  const initialWitness = context.getInitialWitness();
  const acvmCallback = new Oracle(context);
  const { partialWitness } = await acvm(await AcirSimulator.getSolver(), acir, initialWitness, acvmCallback).catch(
    (err: Error) => {
      throw new ExecutionError(
        err.message,
        {
          contractAddress,
          functionSelector,
        },
        extractCallStack(err, abi.debug),
        { cause: err },
      );
    },
  );

  const publicInputs = extractPrivateCircuitPublicInputs(partialWitness, acir);

  const encryptedLogs = context.getEncryptedLogs();
  const unencryptedLogs = context.getUnencryptedLogs();
  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165) --> set this in Noir
  publicInputs.encryptedLogsHash = to2Fields(encryptedLogs.hash());
  publicInputs.encryptedLogPreimagesLength = new Fr(encryptedLogs.getSerializedLength());
  publicInputs.unencryptedLogsHash = to2Fields(unencryptedLogs.hash());
  publicInputs.unencryptedLogPreimagesLength = new Fr(unencryptedLogs.getSerializedLength());

  const callStackItem = new PrivateCallStackItem(contractAddress, functionData, publicInputs, false);
  const returnValues = decodeReturnValues(abi, publicInputs.returnValues);
  const readRequestPartialWitnesses = context.getReadRequestPartialWitnesses(publicInputs.readRequests);
  const newNotes = context.getNewNotes();
  const nestedExecutions = context.getNestedExecutions();
  const enqueuedPublicFunctionCalls = context.getEnqueuedPublicFunctionCalls();

  log(`Returning from call to ${contractAddress.toString()}:${functionSelector}`);

  return {
    acir,
    partialWitness,
    callStackItem,
    returnValues,
    readRequestPartialWitnesses,
    newNotes,
    vk: Buffer.from(abi.verificationKey!, 'hex'),
    nestedExecutions,
    enqueuedPublicFunctionCalls,
    encryptedLogs,
    unencryptedLogs,
  };
}
