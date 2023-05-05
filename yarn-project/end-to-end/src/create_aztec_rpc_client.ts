import { AztecNode } from '@aztec/aztec-node';
import { createAztecRPCServer } from '@aztec/aztec.js';

/**
 * Test helper to create an Aztec RPC server and then add a number of new accounts.
 * @param numberOfAccounts - The number of new accounts to be created once the RPC server is initiated.
 * @param aztecNode - An instance of an Aztec Node.
 * @returns An instance of the Aztec RPC server with some newly created accounts.
 */
export async function createAztecRpcServer(numberOfAccounts = 1, aztecNode: AztecNode) {
  const arc = await createAztecRPCServer(aztecNode);

  for (let i = 0; i < numberOfAccounts; ++i) {
    await arc.addAccount();
  }

  return arc;
}
