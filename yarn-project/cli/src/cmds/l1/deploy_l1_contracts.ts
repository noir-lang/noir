import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { deployAztecContracts } from '../../utils/aztec.js';

export async function deployL1Contracts(
  rpcUrl: string,
  chainId: number,
  privateKey: string,
  mnemonic: string,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  const { l1ContractAddresses } = await deployAztecContracts(rpcUrl, chainId, privateKey, mnemonic, debugLogger);

  log('\n');
  log(`Rollup Address: ${l1ContractAddresses.rollupAddress.toString()}`);
  log(`Registry Address: ${l1ContractAddresses.registryAddress.toString()}`);
  log(`L1 -> L2 Inbox Address: ${l1ContractAddresses.inboxAddress.toString()}`);
  log(`L2 -> L1 Outbox Address: ${l1ContractAddresses.outboxAddress.toString()}`);
  log(`Availability Oracle Address: ${l1ContractAddresses.availabilityOracleAddress.toString()}`);
  log(`Gas Token Address: ${l1ContractAddresses.gasTokenAddress.toString()}`);
  log(`Gas Portal Address: ${l1ContractAddresses.gasPortalAddress.toString()}`);
  log('\n');
}
