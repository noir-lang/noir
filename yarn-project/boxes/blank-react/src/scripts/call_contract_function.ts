import { getWallet } from './util.js';
import { AztecAddress, CompleteAddress, Contract, ContractArtifact, FieldsOf, PXE, TxReceipt } from '@aztec/aztec.js';

export async function callContractFunction(
  address: AztecAddress,
  artifact: ContractArtifact,
  functionName: string,
  typedArgs: any[], // for the exposed functions, this is an array of field elements Fr[]
  pxe: PXE,
  wallet: CompleteAddress,
): Promise<FieldsOf<TxReceipt>> {
  // selectedWallet is how we specify the "sender" of the transaction
  const selectedWallet = await getWallet(wallet, pxe);

  // TODO: switch to the generated typescript class?
  const contract = await Contract.at(address, artifact, selectedWallet);

  return contract.methods[functionName](...typedArgs)
    .send()
    .wait();
}
