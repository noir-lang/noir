import { type Fr } from '@aztec/foundation/fields';

/**
 * A type which along with public key forms a preimage of a contract address. See the link below for more details
 * https://github.com/AztecProtocol/aztec-packages/blob/master/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
 */
export type PartialAddress = Fr;
