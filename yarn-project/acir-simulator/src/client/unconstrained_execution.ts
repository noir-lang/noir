import { FunctionData } from '@aztec/circuits.js';
import { DecodedReturn, decodeReturnValues } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import { extractReturnWitness } from '../acvm/deserialize.js';
import { ACVMField, Oracle, acvm, extractCallStack, fromACVMField, toACVMWitness } from '../acvm/index.js';
import { ExecutionError } from '../common/errors.js';
import { AcirSimulator } from '../index.js';
import { FunctionArtifactWithDebugMetadata } from './db_oracle.js';
import { ViewDataOracle } from './view_data_oracle.js';

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

  const acir = Buffer.from(artifact.bytecode, 'base64');
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

  const returnValues: ACVMField[] = extractReturnWitness(acir, partialWitness);
  return decodeReturnValues(artifact, returnValues.map(fromACVMField));
}
