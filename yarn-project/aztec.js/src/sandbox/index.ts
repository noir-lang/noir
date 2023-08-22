import { Fr, PrivateKey } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { sleep } from '@aztec/foundation/sleep';

import SchnorrAccountContractAbi from '../abis/schnorr_account_contract.json' assert { type: 'json' };
import { AztecRPC, getAccountWallets, getSchnorrAccount } from '../index.js';

export const INITIAL_SANDBOX_ENCRYPTION_KEYS = [
  new PrivateKey(Buffer.from('b2803ec899f76f6b2ac011480d24028f1a29587f8a3a92f7ee9d48d8c085c284', 'hex')),
  new PrivateKey(Buffer.from('6bb46e9a80da2ff7bfff71c2c50eaaa4b15f7ed5ad1ade4261b574ef80b0cdb0', 'hex')),
  new PrivateKey(Buffer.from('0f6addf0da06c33293df974a565b03d1ab096090d907d98055a8b7f4954e120c', 'hex')),
];

export const INITIAL_SANDBOX_SIGNING_KEYS = INITIAL_SANDBOX_ENCRYPTION_KEYS;

export const INITIAL_SANDBOX_SALTS = [Fr.ZERO, Fr.ZERO, Fr.ZERO];

export const INITIAL_SANDBOX_ACCOUNT_CONTRACT_ABI = SchnorrAccountContractAbi;

/**
 * Gets a wallet for the Aztec accounts that are initially stored in the sandbox.
 * @param aztecRpc - An instance of the Aztec RPC interface.
 * @returns An AccountWallet implementation that includes all the accounts found.
 */
export async function getSandboxAccountsWallet(aztecRpc: AztecRPC) {
  return await getAccountWallets(
    aztecRpc,
    INITIAL_SANDBOX_ACCOUNT_CONTRACT_ABI as unknown as ContractAbi,
    INITIAL_SANDBOX_ENCRYPTION_KEYS,
    INITIAL_SANDBOX_SIGNING_KEYS,
    INITIAL_SANDBOX_SALTS,
  );
}

/**
 * Deploys the initial set of schnorr signature accounts to the sandbox
 * @param aztecRpc - An instance of the Aztec RPC interface.
 * @returns The set of deployed Account objects and associated private encryption keys
 */
export async function deployInitialSandboxAccounts(aztecRpc: AztecRPC) {
  const accounts = INITIAL_SANDBOX_ENCRYPTION_KEYS.map((privateKey, i) => {
    const account = getSchnorrAccount(aztecRpc, privateKey, INITIAL_SANDBOX_SIGNING_KEYS[i], INITIAL_SANDBOX_SALTS[i]);
    return {
      account,
      privateKey,
    };
  });
  // Attempt to get as much parallelism as possible
  const deployMethods = await Promise.all(
    accounts.map(async x => {
      const deployMethod = await x.account.getDeployMethod();
      await deployMethod.create({ contractAddressSalt: x.account.salt });
      await deployMethod.simulate({});
      return deployMethod;
    }),
  );
  // Send tx together to try and get them in the same rollup
  const sentTxs = deployMethods.map(dm => {
    return dm.send();
  });
  await Promise.all(
    sentTxs.map(async (tx, i) => {
      const wallet = await accounts[i].account.getWallet();
      return tx.wait({ wallet });
    }),
  );
  return accounts;
}

/**
 * Function to wait until the sandbox becomes ready for use.
 * @param rpcServer - The rpc client connected to the sandbox.
 */
export const waitForSandbox = async (rpcServer: AztecRPC) => {
  while (true) {
    try {
      await rpcServer.getNodeInfo();
      break;
    } catch (err) {
      await sleep(1000);
    }
  }
};
