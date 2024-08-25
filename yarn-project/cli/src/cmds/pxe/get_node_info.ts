import { createCompatibleClient } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

export async function getNodeInfo(rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const info = await client.getNodeInfo();
  log(`Node Version: ${info.nodeVersion}`);
  log(`Chain Id: ${info.l1ChainId}`);
  log(`Protocol Version: ${info.protocolVersion}`);
  log(`Node ENR: ${info.enr}`);
  log(`L1 Contract Addresses:`);
  log(` Rollup Address: ${info.l1ContractAddresses.rollupAddress.toString()}`);
  log(` Registry Address: ${info.l1ContractAddresses.registryAddress.toString()}`);
  log(` L1 -> L2 Inbox Address: ${info.l1ContractAddresses.inboxAddress.toString()}`);
  log(` L2 -> L1 Outbox Address: ${info.l1ContractAddresses.outboxAddress.toString()}`);
  log(` Availability Oracle Address: ${info.l1ContractAddresses.availabilityOracleAddress.toString()}`);
  log(` Fee Juice Address: ${info.l1ContractAddresses.feeJuiceAddress.toString()}`);
  log(` Fee Juice Portal Address: ${info.l1ContractAddresses.feeJuicePortalAddress.toString()}`);

  log(`L2 Contract Addresses:`);
  log(` Class Registerer: ${info.protocolContractAddresses.classRegisterer.toString()}`);
  log(` Fee Juice: ${info.protocolContractAddresses.feeJuice.toString()}`);
  log(` Instance Deployer: ${info.protocolContractAddresses.instanceDeployer.toString()}`);
  log(` Key Registry: ${info.protocolContractAddresses.keyRegistry.toString()}`);
  log(` MultiCall: ${info.protocolContractAddresses.multiCallEntrypoint.toString()}`);
}
