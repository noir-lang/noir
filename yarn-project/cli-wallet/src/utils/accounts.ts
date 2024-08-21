import { getEcdsaRSSHAccount } from '@aztec/accounts/ecdsa';
import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { getIdentities } from '@aztec/accounts/utils';
import { type AztecAddress, Fr, deriveSigningKey } from '@aztec/circuits.js';

import { type PXE } from '../../../circuit-types/src/interfaces/pxe.js';
import { type WalletDB } from '../storage/wallet_db.js';
import { extractECDSAPublicKeyFromBase64String } from './ecdsa.js';

export const AccountTypes = ['schnorr', 'ecdsasecp256r1ssh', 'ecdsasecp256k1'] as const;
export type AccountType = (typeof AccountTypes)[number];

export async function createOrRetrieveAccount(
  pxe: PXE,
  address?: AztecAddress,
  db?: WalletDB,
  type: AccountType = 'schnorr',
  secretKey?: Fr,
  salt?: Fr,
  publicKey?: string | undefined,
) {
  let account;

  salt ??= Fr.ZERO;

  if (db && address) {
    ({ type, secretKey, salt } = db.retrieveAccount(address));
  }

  if (!salt) {
    throw new Error('Cannot retrieve/create wallet without salt');
  }

  if (!secretKey) {
    throw new Error('Cannot retrieve/create wallet without secret key');
  }

  switch (type) {
    case 'schnorr': {
      account = getSchnorrAccount(pxe, secretKey, deriveSigningKey(secretKey), salt);
      break;
    }
    case 'ecdsasecp256r1ssh': {
      let publicSigningKey;
      if (db && address) {
        publicSigningKey = db.retrieveAccountMetadata(address, 'publicSigningKey');
      } else if (publicKey) {
        const identities = await getIdentities();
        const foundIdentity = identities.find(
          identity => identity.type === 'ecdsa-sha2-nistp256' && identity.publicKey === publicKey,
        );
        if (!foundIdentity) {
          throw new Error(`Identity for public key ${publicKey} not found in the SSH agent`);
        }
        publicSigningKey = extractECDSAPublicKeyFromBase64String(foundIdentity.publicKey);
      } else {
        throw new Error('Public key must be provided for ECDSA SSH account');
      }

      account = getEcdsaRSSHAccount(pxe, secretKey, publicSigningKey, salt);
      break;
    }
    default: {
      throw new Error(`Unsupported account type: ${type}`);
    }
  }

  return account;
}
