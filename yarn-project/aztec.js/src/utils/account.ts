import { Fr, PrivateKey, getContractDeploymentInfo } from '@aztec/circuits.js';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi } from '@aztec/foundation/abi';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecRPC, TxStatus } from '@aztec/types';

import { SingleKeyAccountEntrypoint } from '../account/entrypoint/single_key_account_entrypoint.js';
import { EntrypointWallet, Wallet } from '../aztec_rpc_client/wallet.js';
import { ContractDeployer, EntrypointCollection, generatePublicKey } from '../index.js';

/**
 * Creates an Aztec Account.
 * @returns The account's address & public key.
 */
export async function createAccounts(
  aztecRpcClient: AztecRPC,
  accountContractAbi: ContractAbi,
  privateKey?: PrivateKey,
  salt = Fr.random(),
  numberOfAccounts = 1,
  logger = createDebugLogger('aztec:aztec.js:accounts'),
): Promise<Wallet> {
  const accountImpls = new EntrypointCollection();

  for (let i = 0; i < numberOfAccounts; ++i) {
    // TODO(#662): Let the aztec rpc server generate the keypair rather than hardcoding the private key
    const privKey = i == 0 && privateKey ? privateKey : PrivateKey.random();
    const publicKey = await generatePublicKey(privKey);
    const deploymentInfo = await getContractDeploymentInfo(accountContractAbi, [], salt, publicKey);
    await aztecRpcClient.addAccount(privKey, deploymentInfo.address, deploymentInfo.partialAddress);
    const contractDeployer = new ContractDeployer(accountContractAbi, aztecRpcClient, publicKey);
    const tx = contractDeployer.deploy().send({ contractAddressSalt: salt });
    await tx.isMined({ interval: 0.5 });
    const receipt = await tx.getReceipt();
    if (receipt.status !== TxStatus.MINED) {
      throw new Error(`Deployment tx not mined (status is ${receipt.status})`);
    }
    const address = receipt.contractAddress!;
    if (!address.equals(deploymentInfo.address)) {
      throw new Error(
        `Deployment address does not match for account contract (expected ${deploymentInfo.address.toString()} got ${address.toString()})`,
      );
    }
    logger(`Created account ${address.toString()} with public key ${publicKey.toString()}`);
    accountImpls.registerAccount(
      address,
      new SingleKeyAccountEntrypoint(address, deploymentInfo.partialAddress, privKey, await Schnorr.new()),
    );
  }
  return new EntrypointWallet(aztecRpcClient, accountImpls);
}

/**
 * Gets the Aztec accounts that are stored in an Aztec RPC instance.
 * @param aztecRpcClient - An instance of the Aztec RPC interface.
 * @param numberOfAccounts - The number of accounts to fetch.
 * @returns An AccountWallet implementation that includes all the accounts found.
 */
export async function getAccountWallet(
  aztecRpcClient: AztecRPC,
  accountContractAbi: ContractAbi,
  privateKey: PrivateKey,
  salt: Fr,
) {
  const accountCollection = new EntrypointCollection();
  const publicKey = await generatePublicKey(privateKey);
  const deploymentInfo = await getContractDeploymentInfo(accountContractAbi, [], salt, publicKey);
  const address = deploymentInfo.address;

  accountCollection.registerAccount(
    address,
    new SingleKeyAccountEntrypoint(address, deploymentInfo.partialAddress, privateKey, await Schnorr.new()),
  );
  return new EntrypointWallet(aztecRpcClient, accountCollection);
}
