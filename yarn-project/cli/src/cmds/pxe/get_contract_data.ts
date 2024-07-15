import { type AztecAddress } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';

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

  const isPrivatelyDeployed = !!instance;
  const isPubliclyDeployed = await client.isContractPubliclyDeployed(contractAddress);
  if (isPubliclyDeployed && isPrivatelyDeployed) {
    log(`Contract is publicly deployed at ${contractAddress.toString()}`);
  } else if (isPrivatelyDeployed) {
    log(`Contract is registered in the local pxe at ${contractAddress.toString()} but not publicly deployed`);
  } else if (isPubliclyDeployed) {
    log(`Contract is publicly deployed at ${contractAddress.toString()} but not registered in the local pxe`);
  } else {
    log(`No contract found at ${contractAddress.toString()}`);
  }

  if (instance) {
    log(``);
    Object.entries(instance).forEach(([key, value]) => {
      const capitalized = key.charAt(0).toUpperCase() + key.slice(1);
      log(`${capitalized}: ${value.toString()}`);
    });

    if (contractClass) {
      log(`\nBytecode: ${contractClass.packedBytecode.toString('base64')}`);
    }
    log('');
  }
}
