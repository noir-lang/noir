import { type AuthWitness, type PXE } from '@aztec/circuit-types';

import { type IntentAction, type IntentInnerHash } from '../utils/authwit.js';
import { type AccountInterface, type AccountKeyRotationInterface } from './interface.js';

/**
 * The wallet interface.
 */
export type Wallet = AccountInterface &
  PXE &
  AccountKeyRotationInterface & {
    createAuthWit(intent: IntentInnerHash | IntentAction): Promise<AuthWitness>;
  };
