import { type FunctionData } from '@aztec/circuits.js';
import { type DecodedReturn, type FunctionArtifactWithDebugMetadata, decodeReturnValues } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import { extractReturnWitness } from '../acvm/deserialize.js';
import { Oracle, acvm, extractCallStack, toACVMWitness } from '../acvm/index.js';
import { ExecutionError } from '../common/errors.js';
import { AcirSimulator } from './simulator.js';
import { type ViewDataOracle } from './view_data_oracle.js';

// docs:start:execute_unconstrained_function
/**
 * Execute an unconstrained function and return the decoded values.
 */
export async function executeUnconstrainedFunction(
  oracle: ViewDataOracle,
  artifact: FunctionArtifactWithDebugMetadata,
  contractAddress: AztecAddress,
  functionData: FunctionData,
  args: Fr[],
  log = createDebugLogger('aztec:simulator:unconstrained_execution'),
): Promise<DecodedReturn> {
  const functionSelector = functionData.selector;
  log(`Executing unconstrained function ${contractAddress}:${functionSelector}`);

  const acir = artifact.bytecode;
  const initialWitness = toACVMWitness(0, args);
  const { partialWitness } = await acvm(
    await AcirSimulator.getSolver(),
    acir,
    initialWitness,
    new Oracle(oracle),
  ).catch((err: Error) => {
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

  return decodeReturnValues(artifact, extractReturnWitness(acir, partialWitness));
}
// docs:end:execute_unconstrained_function
