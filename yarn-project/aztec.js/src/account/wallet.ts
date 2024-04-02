import { type PXE } from '@aztec/circuit-types';

import { type AccountInterface } from './interface.js';

/**
 * The wallet interface.
 */
export type Wallet = AccountInterface & PXE;
