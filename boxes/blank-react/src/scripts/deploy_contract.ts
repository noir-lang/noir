import { AztecAddress, CompleteAddress, Contract, ContractArtifact, DeployMethod, Fr, PXE } from '@aztec/aztec.js';

export async function deployContract(
  activeWallet: CompleteAddress,
  contractArtifact: ContractArtifact,
  typedArgs: Fr[], // encode prior to passing in
  salt: Fr,
  pxe: PXE,
): Promise<AztecAddress> {
  const tx = new DeployMethod(
    activeWallet.publicKey,
    pxe,
    contractArtifact,
    (a, w) => Contract.at(a, contractArtifact, w),
    typedArgs,
  ).send({
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
