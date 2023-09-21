import { AztecAddress, AztecRPC, CompleteAddress, Contract } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { getWallet } from './util.js';

export async function viewContractFunction(
  address: AztecAddress,
  abi: ContractAbi,
  functionName: string,
  typedArgs: any[],
  rpc: AztecRPC,
  wallet: CompleteAddress,
) {
  // we specify the account that is calling the view function by passing in the wallet to the Contract
  const selectedWallet = await getWallet(wallet, rpc);
  const contract = await Contract.at(address, abi, selectedWallet);

  const viewResult = await contract.methods[functionName](...typedArgs).view({ from: wallet.address });
  return viewResult;
}
