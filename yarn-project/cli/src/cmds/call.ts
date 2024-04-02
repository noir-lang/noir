import { type AztecAddress } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { format } from 'util';

import { createCompatibleClient } from '../client.js';
import { getFunctionArtifact, getTxSender, prepTx } from '../utils.js';

export async function call(
  functionName: string,
  functionArgsIn: any[],
  contractArtifactPath: string,
  contractAddress: AztecAddress,
  fromAddress: string | undefined,
  rpcUrl: string,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const { functionArgs, contractArtifact } = await prepTx(contractArtifactPath, functionName, functionArgsIn, log);

  const fnArtifact = getFunctionArtifact(contractArtifact, functionName);
  if (fnArtifact.parameters.length !== functionArgs.length) {
    throw Error(
      `Invalid number of args passed. Expected ${fnArtifact.parameters.length}; Received: ${functionArgs.length}`,
    );
  }

  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const from = await getTxSender(client, fromAddress);
  const result = await client.viewTx(functionName, functionArgs, contractAddress, from);
  log(format('\nView result: ', result, '\n'));
}
