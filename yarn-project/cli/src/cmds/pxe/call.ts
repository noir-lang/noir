import { type AztecAddress, ContractFunctionInteraction, SignerlessWallet } from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { format } from 'util';

import { createCompatibleClient } from '../../client.js';
import { getFunctionArtifact, prepTx } from '../../utils/aztec.js';
import { getTxSender } from '../../utils/commands.js';

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
  const { l1ChainId: chainId, protocolVersion } = await client.getNodeInfo();
  const call = new ContractFunctionInteraction(
    new SignerlessWallet(client, new DefaultMultiCallEntrypoint(chainId, protocolVersion)),
    contractAddress,
    fnArtifact,
    functionArgs,
  );
  const from = await getTxSender(client, fromAddress);
  const result = await call.simulate({ from });
  log(format('\nView result: ', result, '\n'));
}
