import { AztecRPC, TxStatus, getContractDeploymentInfo } from '@aztec/aztec-rpc';
import { AztecAddress, CircuitsWasm, Fr, Point } from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import { EcdsaAccountContractAbi } from '@aztec/noir-contracts/examples';
import { AccountWallet, Wallet } from '../aztec_rpc_client/wallet.js';
import {
  AccountCollection,
  AccountContract,
  ContractDeployer,
  EcdsaAuthProvider,
  generatePublicKey,
} from '../index.js';

/**
 * Creates an Aztec Account.
 * @returns The account's address & public key.
 */
export async function createAccounts(
  aztecRpcClient: AztecRPC,
  privateKey?: Buffer,
  numberOfAccounts = 1,
  logger = createDebugLogger('aztec:aztec.js:accounts'),
): Promise<Wallet> {
  const accountImpls = new AccountCollection();
  const results: [AztecAddress, Point][] = [];
  const wasm = await CircuitsWasm.get();
  for (let i = 0; i < numberOfAccounts; ++i) {
    // TODO(#662): Let the aztec rpc server generate the keypair rather than hardcoding the private key
    const privKey = i == 0 && privateKey ? privateKey : randomBytes(32);
    const accountAbi = EcdsaAccountContractAbi;
    const publicKey = await generatePublicKey(privKey);
    const salt = Fr.random();
    const deploymentInfo = await getContractDeploymentInfo(accountAbi, [], salt, publicKey);
    await aztecRpcClient.addAccount(privKey, deploymentInfo.address, deploymentInfo.partialAddress, accountAbi);
    const contractDeployer = new ContractDeployer(accountAbi, aztecRpcClient, publicKey);
    const tx = contractDeployer.deploy().send({ contractAddressSalt: salt });
    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();
    if (receipt.status !== TxStatus.MINED) {
      throw new Error(`Deployment tx not mined (status is ${receipt.status})`);
    }
    const address = receipt.contractAddress!;
    logger(`Created account ${address.toString()} with public key ${publicKey.toString()}`);
    accountImpls.registerAccount(
      address,
      new AccountContract(
        address,
        publicKey,
        new EcdsaAuthProvider(privKey),
        deploymentInfo.partialAddress,
        accountAbi,
        wasm,
      ),
    );
    results.push([address, publicKey]);
  }
  return new AccountWallet(aztecRpcClient, accountImpls);
}
