import { type AztecAddress } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function getContractData(
  rpcUrl: string,
  contractAddress: AztecAddress,
  includeBytecode: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const instance = await client.getContractInstance(contractAddress);
  const contractClass = includeBytecode && instance && (await client.getContractClass(instance?.contractClassId));

  if (!instance) {
    log(`No contract found at ${contractAddress}`);
    return;
  }

  log(`\nContract Data:`);
  Object.entries(instance).forEach(([key, value]) => {
    const capitalized = key.charAt(0).toUpperCase() + key.slice(1);
    log(`${capitalized}: ${value.toString()}`);
  });

  if (contractClass) {
    log(`Bytecode: ${contractClass.packedBytecode.toString('base64')}`);
  }
  log('\n');
}
