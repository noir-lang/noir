import { AztecAddress } from '@aztec/aztec.js';
import { ContractData } from '@aztec/circuit-types';
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

/**
 *
 */
export async function getContractData(
  rpcUrl: string,
  contractAddress: AztecAddress,
  includeBytecode: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const contractDataWithOrWithoutBytecode = includeBytecode
    ? await client.getExtendedContractData(contractAddress)
    : await client.getContractData(contractAddress);

  if (!contractDataWithOrWithoutBytecode) {
    log(`No contract data found at ${contractAddress}`);
    return;
  }
  let contractData: ContractData;

  if ('contractData' in contractDataWithOrWithoutBytecode) {
    contractData = contractDataWithOrWithoutBytecode.contractData;
  } else {
    contractData = contractDataWithOrWithoutBytecode;
  }
  log(`\nContract Data: \nAddress: ${contractData.contractAddress.toString()}`);
  log(`Portal:  ${contractData.portalContractAddress.toString()}`);
  if ('bytecode' in contractDataWithOrWithoutBytecode) {
    log(`Bytecode: ${contractDataWithOrWithoutBytecode.bytecode}`);
  }
  log('\n');
}
