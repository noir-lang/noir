import { getWallet } from './util.js';
import { AztecAddress, PXE, CompleteAddress, Contract } from '@aztec/aztec.js';
import { ContractArtifact } from '@aztec/foundation/abi';

export async function viewContractFunction(
  address: AztecAddress,
  artifact: ContractArtifact,
  functionName: string,
  typedArgs: any[],
  pxe: PXE,
  wallet: CompleteAddress,
) {
  // we specify the account that is calling the view function by passing in the wallet to the Contract
  const selectedWallet = await getWallet(wallet, pxe);
  const contract = await Contract.at(address, artifact, selectedWallet);

  return await contract.methods[functionName](...typedArgs).view({ from: wallet.address });
}
