import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AztecAddress, ContractFunctionInteraction, SignerlessWallet } from '@aztec/aztec.js';
import { createCompatibleClient } from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { Fr, deriveSigningKey } from '@aztec/circuits.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { format } from 'util';

import { getFunctionArtifact, prepTx } from '../../utils/aztec.js';

export async function call(
  functionName: string,
  functionArgsIn: any[],
  contractArtifactPath: string,
  contractAddress: AztecAddress,
  privateKey: Fr | undefined,
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
  const wallet = privateKey
    ? await getSchnorrAccount(client, privateKey, deriveSigningKey(privateKey), Fr.ZERO).getWallet()
    : new SignerlessWallet(client, new DefaultMultiCallEntrypoint(chainId, protocolVersion));

  const call = new ContractFunctionInteraction(wallet, contractAddress, fnArtifact, functionArgs);
  const result = await call.simulate();

  log(format('\nView result: ', result, '\n'));
}
