import { createAztecRpcClient } from '@aztec/aztec.js';
import { makeFetch } from '@aztec/foundation/json-rpc/client';

const retries = [1, 1, 2];

/**
 * Creates an Aztec RPC client with a given set of retries on non-server errors.
 * @param rpcUrl - URL of the RPC server.
 * @returns An RPC client.
 */
export function createClient(rpcUrl: string) {
  const fetch = makeFetch(retries, true);
  return createAztecRpcClient(rpcUrl, fetch);
}
