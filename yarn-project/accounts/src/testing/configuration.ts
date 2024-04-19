import { generatePublicKey } from '@aztec/aztec.js';
import { type AccountWalletWithSecretKey } from '@aztec/aztec.js/wallet';
import { type PXE } from '@aztec/circuit-types';
import { GeneratorIndex } from '@aztec/circuits.js/constants';
import { sha512ToGrumpkinScalar } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';

import { getSchnorrAccount } from '../schnorr/index.js';

export const INITIAL_TEST_SECRET_KEYS = [
  Fr.fromString('2153536ff6628eee01cf4024889ff977a18d9fa61d0e414422f7681cf085c281'),
  Fr.fromString('aebd1b4be76efa44f5ee655c20bf9ea60f7ae44b9a7fd1fd9f189c7a0b0cdae'),
  Fr.fromString('0f6addf0da06c33293df974a565b03d1ab096090d907d98055a8b7f4954e120c'),
];

export const INITIAL_TEST_ENCRYPTION_KEYS = INITIAL_TEST_SECRET_KEYS.map(secretKey =>
  sha512ToGrumpkinScalar([secretKey, GeneratorIndex.IVSK_M]),
);
// TODO(#5837): come up with a standard signing key derivation scheme instead of using ivsk_m as signing keys here
export const INITIAL_TEST_SIGNING_KEYS = INITIAL_TEST_ENCRYPTION_KEYS;

export const INITIAL_TEST_ACCOUNT_SALTS = [Fr.ZERO, Fr.ZERO, Fr.ZERO];

/**
 * Gets a collection of wallets for the Aztec accounts that are initially stored in the test environment.
 * @param pxe - PXE instance.
 * @returns A set of AccountWallet implementations for each of the initial accounts.
 */
export function getInitialTestAccountsWallets(pxe: PXE): Promise<AccountWalletWithSecretKey[]> {
  return Promise.all(
    INITIAL_TEST_SECRET_KEYS.map((encryptionKey, i) =>
      getSchnorrAccount(pxe, encryptionKey!, INITIAL_TEST_SIGNING_KEYS[i]!, INITIAL_TEST_ACCOUNT_SALTS[i]).getWallet(),
    ),
  );
}

/**
 * Queries a PXE for it's registered accounts and returns wallets for those accounts using keys in the initial test accounts.
 * @param pxe - PXE instance.
 * @returns A set of AccountWallet implementations for each of the initial accounts.
 */
export async function getDeployedTestAccountsWallets(pxe: PXE): Promise<AccountWalletWithSecretKey[]> {
  const registeredAccounts = await pxe.getRegisteredAccounts();
  return Promise.all(
    INITIAL_TEST_SECRET_KEYS.filter(initialSecretKey => {
      const initialEncryptionKey = sha512ToGrumpkinScalar([initialSecretKey, GeneratorIndex.IVSK_M]);
      const publicKey = generatePublicKey(initialEncryptionKey);
      return registeredAccounts.find(registered => registered.publicKey.equals(publicKey)) != undefined;
    }).map(secretKey => {
      const signingKey = sha512ToGrumpkinScalar([secretKey, GeneratorIndex.IVSK_M]);
      // TODO(#5726): use actual salt here instead of hardcoding Fr.ZERO
      return getSchnorrAccount(pxe, secretKey, signingKey, Fr.ZERO).getWallet();
    }),
  );
}

/**
 * Deploys the initial set of schnorr signature accounts to the test environment
 * @param pxe - PXE instance.
 * @returns The set of deployed Account objects and associated private encryption keys
 */
export async function deployInitialTestAccounts(pxe: PXE) {
  const accounts = INITIAL_TEST_SECRET_KEYS.map((secretKey, i) => {
    const account = getSchnorrAccount(pxe, secretKey, INITIAL_TEST_SIGNING_KEYS[i], INITIAL_TEST_ACCOUNT_SALTS[i]);
    return {
      account,
      secretKey,
    };
  });
  // Attempt to get as much parallelism as possible
  const deployMethods = await Promise.all(
    accounts.map(async x => {
      const deployMethod = await x.account.getDeployMethod();
      await deployMethod.create({
        contractAddressSalt: x.account.salt,
        skipClassRegistration: true,
        skipPublicDeployment: true,
        universalDeploy: true,
      });
      await deployMethod.prove({});
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
