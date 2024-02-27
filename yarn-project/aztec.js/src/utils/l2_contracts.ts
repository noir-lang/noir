import { PXE } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/foundation/aztec-address';

/**
 * Checks whether a give contract is deployed on the network.
 * @param pxe - The PXE to use to obtain the information.
 * @param contractAddress - The address of the contract to check.
 * @returns A flag indicating whether the contract is deployed.
 */
export async function isContractDeployed(pxe: PXE, contractAddress: AztecAddress): Promise<boolean> {
  return !!(await pxe.getContractData(contractAddress)) || !!(await pxe.getContractInstance(contractAddress));
}
