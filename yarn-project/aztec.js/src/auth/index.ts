import { AztecAddress, Fr } from '@aztec/circuits.js';
import { EntrypointPayload } from '../account_impl/account_contract.js';

export * from './ecdsa.js';
export * from './schnorr.js';

/**
 * An interface for the payload returned from auth operations.
 */
export interface AuthPayload {
  toBuffer(): Buffer;
  toFields(): Fr[];
}

/**
 * A dummy implementation of the auth provider
 */
export class DummyAuthProvider {
  authenticateTx(_payload: EntrypointPayload, _payloadHash: Buffer, _address: AztecAddress): Promise<AuthPayload> {
    return Promise.resolve({
      toBuffer: () => Buffer.alloc(0),
      toFields: () => [],
    } as AuthPayload);
  }
}
