import { AztecAddress, AztecRPC, CompleteAddress, Contract, TxReceipt } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { FieldsOf } from '@aztec/foundation/types';
import { getWallet } from './util.js';

export async function callContractFunction(
  address: AztecAddress,
  abi: ContractAbi,
  functionName: string,
  typedArgs: any[], // for the exposed functions, this is an array of field elements Fr[]
  rpc: AztecRPC,
  wallet: CompleteAddress,
): Promise<FieldsOf<TxReceipt>> {
  // selectedWallet is how we specify the "sender" of the transaction
  const selectedWallet = await getWallet(wallet, rpc);

  // TODO: switch to the generated typescript class?
  const contract = await Contract.at(address, abi, selectedWallet);

  const returnVal = contract.methods[functionName](...typedArgs)
    .send()
    .wait();

  return await returnVal;
}
