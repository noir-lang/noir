import { type PXE } from '@aztec/circuit-types';
import { type Fr } from '@aztec/circuits.js';

import { type Salt } from '../account/index.js';
import { type AccountInterface } from '../account/interface.js';
import { AccountWallet } from './account_wallet.js';

/**
 * Extends {@link AccountWallet} with the encryption private key. Not required for
 * implementing the wallet interface but useful for testing purposes or exporting
 * an account to another pxe.
 */
export class AccountWalletWithSecretKey extends AccountWallet {
  constructor(
    pxe: PXE,
    account: AccountInterface,
    private secretKey: Fr,
    /** Deployment salt for this account contract. */
    public readonly salt: Salt,
  ) {
    super(pxe, account);
  }

  /** Returns the encryption private key associated with this account. */
  public getSecretKey() {
    return this.secretKey;
  }
}
