import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { Fr, deriveSigningKey } from '@aztec/circuits.js';

import { type PXE } from '../../../circuit-types/src/interfaces/pxe.js';
import { type WalletDB } from '../storage/wallet_db.js';

export enum AccountType {
  SCHNORR = 'schnorr',
  ECDSASECP256R1 = 'ecdsasecp256r1',
  ECDSASECP256K1 = 'ecdsasecp256k1',
}

export async function createOrRetrieveWallet(
  type: AccountType,
  pxe: PXE,
  privateKey: Fr | undefined,
  aliasOrAddress: string | undefined,
  db?: WalletDB,
) {
  let wallet;
  if (db && aliasOrAddress) {
    const { salt, privateKey } = db.retrieveAccount(aliasOrAddress);
    wallet = await getSchnorrAccount(pxe, privateKey, deriveSigningKey(privateKey), salt).getWallet();
  } else if (privateKey) {
    wallet = await getSchnorrAccount(pxe, privateKey, deriveSigningKey(privateKey), Fr.ZERO).getWallet();
  } else {
    throw new Error('Either a private key or an account address/alias must be provided');
  }
  return wallet;
}
