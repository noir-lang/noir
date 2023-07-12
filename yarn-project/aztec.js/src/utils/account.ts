import { CircuitsWasm, Fr, getContractDeploymentInfo } from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecRPC, TxStatus } from '@aztec/types';
import { SchnorrAccountContractAbi } from '@aztec/noir-contracts/examples';
import { Schnorr } from '@aztec/circuits.js/barretenberg';

import { AccountWallet, Wallet } from '../aztec_rpc_client/wallet.js';
import {
  AccountCollection,
  AccountContract,
  ContractDeployer,
  SchnorrAuthProvider,
  generatePublicKey,
} from '../index.js';

/**
 * Creates an Aztec Account.
 * @returns The account's address & public key.
 */
export async function createAccounts(
  aztecRpcClient: AztecRPC,
  privateKey?: Buffer,
  salt = Fr.random(),
  numberOfAccounts = 1,
  logger = createDebugLogger('aztec:aztec.js:accounts'),
): Promise<Wallet> {
  const accountAbi = SchnorrAccountContractAbi;
  const accountImpls = new AccountCollection();
  const wasm = await CircuitsWasm.get();
  for (let i = 0; i < numberOfAccounts; ++i) {
    // TODO(#662): Let the aztec rpc server generate the keypair rather than hardcoding the private key
    const privKey = i == 0 && privateKey ? privateKey : randomBytes(32);
    const publicKey = await generatePublicKey(privKey);
    const deploymentInfo = await getContractDeploymentInfo(accountAbi, [], salt, publicKey);
    await aztecRpcClient.addAccount(privKey, deploymentInfo.address, deploymentInfo.partialAddress, accountAbi);
    const contractDeployer = new ContractDeployer(accountAbi, aztecRpcClient, publicKey);
    const tx = contractDeployer.deploy().send({ contractAddressSalt: salt });
    await tx.isMined(0, 0.5);
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
      new AccountContract(
        address,
        publicKey,
        new SchnorrAuthProvider(await Schnorr.new(), privKey),
        deploymentInfo.partialAddress,
        accountAbi,
        wasm,
      ),
    );
  }
  return new AccountWallet(aztecRpcClient, accountImpls);
}

/**
 * Gets the Aztec accounts that are stored in an Aztec RPC instance.
 * @param aztecRpcClient - An instance of the Aztec RPC interface.
 * @param numberOfAccounts - The number of accounts to fetch.
 * @returns An AccountWallet implementation that includes all the accounts found.
 */
export async function getAccountWallet(aztecRpcClient: AztecRPC, privateKey: Buffer, salt: Fr) {
  const wasm = await CircuitsWasm.get();
  const accountCollection = new AccountCollection();
  const publicKey = await generatePublicKey(privateKey);
  const address = await aztecRpcClient.getAccountAddress(publicKey);
  const deploymentInfo = await getContractDeploymentInfo(SchnorrAccountContractAbi, [], salt, publicKey);

  accountCollection.registerAccount(
    address,
    new AccountContract(
      address,
      publicKey,
      new SchnorrAuthProvider(await Schnorr.new(), privateKey),
      deploymentInfo.partialAddress,
      SchnorrAccountContractAbi,
      wasm,
    ),
  );
  return new AccountWallet(aztecRpcClient, accountCollection);
}
