import { AztecAddress, Fr } from '@aztec/circuits.js';
import { EntrypointPayload } from '../account_impl/account_contract.js';

export * from './ecdsa.js';

/**
 * An interface for the payload returned from auth operations.
 */
export interface AuthPayload {
  toBuffer(): Buffer;
  toFields(): Fr[];
}

/**
 * The interface for an auth operations provider.
 * TODO: How coupled should the auth provider be a to a specific account contract implementation? Can we remove the dependency on a specific EntrypointPayload format here?
 */
export interface TxAuthProvider {
  authenticateTx(payload: EntrypointPayload, payloadHash: Buffer, address: AztecAddress): Promise<AuthPayload>;
}

/**
 * A dummy implementation of the auth provider
 */
export class DummyAuthProvider implements TxAuthProvider {
  authenticateTx(_payload: EntrypointPayload, _payloadHash: Buffer, _address: AztecAddress): Promise<AuthPayload> {
    return Promise.resolve({
      toBuffer: () => Buffer.alloc(0),
      toFields: () => [],
    } as AuthPayload);
  }
}
