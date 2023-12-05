import { LogFn } from '@aztec/foundation/log';

import { unboxContract } from '../unbox.js';

/**
 *
 */
export async function unbox(contractName: string, localDirectory: string | undefined, cliVersion: string, log: LogFn) {
  const unboxTo: string = localDirectory ? localDirectory : contractName;
  await unboxContract(contractName, unboxTo, cliVersion, log);
}
