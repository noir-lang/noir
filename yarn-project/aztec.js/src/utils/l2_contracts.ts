import { AztecAddress } from '@aztec/foundation/aztec-address';
import { AztecRPC } from '@aztec/types';

/**
 * Checks whether a give contract is deployed on the network.
 * @param aztecRpcClient - The aztec rpc client to use to obtain the information.
 * @param contractAddress - The address of the contract to check.
 * @returns A flag indicating whether the contract is deployed.
 */
export async function isContractDeployed(aztecRpcClient: AztecRPC, contractAddress: AztecAddress): Promise<boolean> {
  return !!(await aztecRpcClient.getContractData(contractAddress));
}
