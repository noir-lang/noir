import { PXE } from '@aztec/circuit-types';

import { AccountInterface } from './interface.js';

/**
 * The wallet interface.
 */
export type Wallet = AccountInterface & PXE;
