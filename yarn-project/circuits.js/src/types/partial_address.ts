import { Fr } from '../index.js';

/**
 * A type which along with public key forms a preimage of a contract address. See the link bellow for more details
 * https://github.com/AztecProtocol/aztec-packages/blob/janb/rpc-interface-cleanup/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
 */
export type PartialAddress = Fr;
