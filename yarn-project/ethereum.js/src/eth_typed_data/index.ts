import { getMessage } from './eip-712.js';
import { TypedData } from './typed_data.js';

export * from './typed_data.js';

/**
 * Computes the hash of the given TypedData object according to EIP-712.
 * This function helps in creating a unique representation of the typed data
 * which can be used for signing or verifying signatures in Ethereum transactions.
 * The input 'data' should be a valid TypedData object containing types, domain, and message information.
 * Throws an error if the input is not a valid TypedData object.
 *
 * @param data - The TypedData object to be hashed.
 * @returns A string representing the hash of the typed data.
 */
export function getTypedDataHash(data: TypedData) {
  return getMessage(data, true);
}
