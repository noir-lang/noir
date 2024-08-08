import { getEcdsaRSSHAccount } from '@aztec/accounts/ecdsa';
import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { getIdentities } from '@aztec/accounts/utils';
import { type AztecAddress, Fr, deriveSigningKey } from '@aztec/circuits.js';

import { type PXE } from '../../../circuit-types/src/interfaces/pxe.js';
import { type WalletDB } from '../storage/wallet_db.js';
import { extractECDSAPublicKeyFromBase64String } from './ecdsa.js';

export const AccountTypes = ['schnorr', 'ecdsasecp256r1ssh', 'ecdsasecp256k1'] as const;
export type AccountType = (typeof AccountTypes)[number];

export async function createAndStoreAccount(
  client: PXE,
  type: AccountType,
  secretKey: Fr,
  publicKey: string | undefined,
  salt: Fr,
  alias: string | undefined,
  db?: WalletDB,
) {
  let account;
  switch (type) {
    case 'schnorr': {
      account = getSchnorrAccount(client, secretKey, deriveSigningKey(secretKey), salt);
      const { address } = account.getCompleteAddress();
      if (db) {
        await db.storeAccount(address, { type, alias, secretKey, salt });
      }
      break;
    }
    case 'ecdsasecp256r1ssh': {
      if (!publicKey) {
        throw new Error('Public key stored in the SSH agent must be provided for ECDSA SSH account');
      }
      const identities = await getIdentities();
      const foundIdentity = identities.find(
        identity => identity.type === 'ecdsa-sha2-nistp256' && identity.publicKey === publicKey,
      );
      if (!foundIdentity) {
        throw new Error(`Identity for public key ${publicKey} not found in the SSH agent`);
      }

      const publicSigningKey = extractECDSAPublicKeyFromBase64String(foundIdentity.publicKey);
      account = getEcdsaRSSHAccount(client, secretKey, publicSigningKey, salt);
      const { address } = account.getCompleteAddress();
      if (db) {
        await db.storeAccount(address, { type, alias, secretKey, salt });
        await db.storeAccountMetadata(address, 'publicSigningKey', publicSigningKey);
      }
      break;
    }
    default: {
      throw new Error(`Unsupported account type: ${type}`);
    }
  }

  return account;
}

export async function createOrRetrieveWallet(
  pxe: PXE,
  address?: AztecAddress,
  type?: AccountType,
  secretKey?: Fr,
  publicKey?: string | undefined,
  db?: WalletDB,
) {
  let wallet, salt;

  if (db && address) {
    ({ type, secretKey, salt } = db.retrieveAccount(address));
  } else {
    type = 'schnorr';
    salt = Fr.ZERO;
  }

  if (!secretKey) {
    throw new Error('Cannot retrieve wallet without secret key');
  }

  switch (type) {
    case 'schnorr': {
      wallet = await getSchnorrAccount(pxe, secretKey, deriveSigningKey(secretKey), salt).getWallet();
      break;
    }
    case 'ecdsasecp256r1ssh': {
      let publicSigningKey;
      if (db && address) {
        publicSigningKey = db.retrieveAccountMetadata(address, 'publicSigningKey');
      } else if (publicKey) {
        publicSigningKey = extractECDSAPublicKeyFromBase64String(publicKey);
      } else {
        throw new Error('Public key must be provided for ECDSA SSH account');
      }
      wallet = await getEcdsaRSSHAccount(pxe, secretKey, publicSigningKey, salt).getWallet();
      break;
    }
    default: {
      throw new Error(`Unsupported account type: ${type}`);
    }
  }

  return wallet;
}
