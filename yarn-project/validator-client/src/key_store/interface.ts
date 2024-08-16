import { type Signature } from '@aztec/circuit-types';

/** Key Store
 *
 * A keystore interface that can be replaced with a local keystore / remote signer service
 */
export interface ValidatorKeyStore {
  sign(message: Buffer): Promise<Signature>;
}
