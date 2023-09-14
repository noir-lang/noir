import { AztecAddress, CompleteAddress, DeployMethod, Fr } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { AztecRPC } from '@aztec/types';

export async function deployContract(
  activeWallet: CompleteAddress,
  contractAbi: ContractAbi,
  typedArgs: Fr[], // encode prior to passing in
  salt: Fr,
  client: AztecRPC,
): Promise<AztecAddress> {
  const tx = new DeployMethod(activeWallet.publicKey, client, contractAbi, typedArgs).send({
    contractAddressSalt: salt,
  });
  await tx.wait();
  const receipt = await tx.getReceipt();
  if (receipt.contractAddress) {
    return receipt.contractAddress;
  } else {
    throw new Error(`Contract not deployed (${receipt.toJSON()})`);
  }
}
