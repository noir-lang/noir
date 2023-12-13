import { getWallet } from './util.js';
import { AztecAddress, CompleteAddress, Contract, ContractArtifact, PXE } from '@aztec/aztec.js';

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
