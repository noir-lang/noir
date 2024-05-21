import { type PXE } from '@aztec/circuit-types';

import { type AccountInterface, type AccountKeyRotationInterface } from './interface.js';

/**
 * The wallet interface.
 */
export type Wallet = AccountInterface & PXE & AccountKeyRotationInterface;
