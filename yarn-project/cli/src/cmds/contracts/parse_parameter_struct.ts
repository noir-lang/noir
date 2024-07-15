import { type StructType } from '@aztec/foundation/abi';
import { JsonStringify } from '@aztec/foundation/json-rpc';
import { type LogFn } from '@aztec/foundation/log';

import { parseStructString } from '../../encoding.js';
import { getContractArtifact } from '../../utils/aztec.js';

export async function parseParameterStruct(
  encodedString: string,
  contractArtifactPath: string,
  parameterName: string,
  log: LogFn,
) {
  const contractArtifact = await getContractArtifact(contractArtifactPath, log);
  const parameterAbitype = contractArtifact.functions
    .map(({ parameters }) => parameters)
    .flat()
    .find(({ name, type }) => name === parameterName && type.kind === 'struct');

  if (!parameterAbitype) {
    log(`No struct parameter found with name ${parameterName}`);
    return;
  }

  const data = parseStructString(encodedString, parameterAbitype.type as StructType);
  log(`\nStruct Data: \n${JsonStringify(data, true)}\n`);
}
