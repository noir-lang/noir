import { AztecAddress, EthAddress, Fr, Header } from '@aztec/circuits.js';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { type ProverNode } from './prover-node.js';

/**
 * Wrap a ProverNode instance with a JSON RPC HTTP server.
 * @param node - The ProverNode
 * @returns An JSON-RPC HTTP server
 */
export function createProverNodeRpcServer(node: ProverNode) {
  const rpc = new JsonRpcServer(
    node,
    {
      AztecAddress,
      EthAddress,
      Fr,
      Header,
    },
    {},
    // disable methods not part of the AztecNode interface
    ['start', 'stop', 'createProvingJob', 'work', 'getProver'],
  );
  return rpc;
}
