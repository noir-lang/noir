import { PXE } from '@aztec/types';

import { AccountInterface } from '../account/index.js';

export * from './base_wallet.js';
export * from './account_wallet.js';
export * from './signerless_wallet.js';

/**
 * The wallet interface.
 */
export type Wallet = AccountInterface & PXE;
